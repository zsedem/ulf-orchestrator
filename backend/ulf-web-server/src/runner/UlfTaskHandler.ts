/**
 * UlfTaskHandler
 *
 * Factory function that creates a Dispatcher-compatible task handler
 * wrapping UlfRunner with LogBroadcaster integration.
 *
 * This is the glue between:
 * - Dispatcher: Executes tasks from the queue
 * - UlfRunner: Spawns and manages ulf child processes
 * - LogBroadcaster: Streams logs to WebSocket clients
 *
 * Design Notes:
 * - Factory pattern keeps UlfRunner decoupled from WebSocket concerns
 * - Each task execution creates a fresh UlfRunner instance
 * - State changes and output are broadcast to subscribed clients
 */

import { QueuedTask, TaskExecutionContext, TaskHandler } from "../queue";
import { UlfRunner, UlfRunnerOptions, RunnerResult } from "./UlfRunner";
import { getLogBroadcaster } from "../api/LogBroadcaster";
import { RunnerState } from "./RunnerState";
import { UlfEventParser } from "./UlfEventParser";

/**
 * Payload expected by the ulf task handler
 */
export interface UlfTaskPayload {
  /** The prompt text to execute */
  prompt: string;
  /** Additional CLI arguments */
  args?: string[];
  /** Working directory override */
  cwd?: string;
  /** Database task ID for broadcasting (allows frontend to subscribe with DB task ID) */
  dbTaskId?: string;
}

/**
 * Options for creating a ulf task handler
 */
export interface UlfTaskHandlerOptions extends Omit<UlfRunnerOptions, "onOutput" | "cwd"> {
  /** Default working directory (can be overridden per-task) */
  defaultCwd?: string;
}

/**
 * Creates a task handler that executes ulf run commands and broadcasts output.
 *
 * @param options - UlfRunner configuration options
 * @returns TaskHandler compatible with Dispatcher.registerHandler()
 *
 * @example
 * ```typescript
 * const dispatcher = new Dispatcher(queue, eventBus);
 * dispatcher.registerHandler('ulf.run', createUlfTaskHandler({
 *   command: 'ulf',
 *   defaultCwd: process.cwd(),
 * }));
 * ```
 */
export function createUlfTaskHandler(
  options: UlfTaskHandlerOptions = {}
): TaskHandler<UlfTaskPayload, RunnerResult> {
  const { defaultCwd, ...runnerOptions } = options;

  return async (task: QueuedTask, context: TaskExecutionContext): Promise<RunnerResult> => {
    const payload = task.payload as unknown as UlfTaskPayload;
    const broadcaster = getLogBroadcaster();

    // Use dbTaskId for broadcasting so frontend can subscribe with database task ID
    // Falls back to queue task ID if dbTaskId not provided (for direct queue usage)
    const broadcastId = payload.dbTaskId || task.id;

    // Create a fresh runner for this task
    // Pass dbTaskId as taskId so ProcessSupervisor can find the process for cancellation
    const runner = new UlfRunner({
      ...runnerOptions,
      cwd: payload.cwd ?? defaultCwd,
      taskId: payload.dbTaskId,
    });

    // Create event parser to detect Ulf events from stdout
    const eventParser = new UlfEventParser((event) => {
      broadcaster.broadcastEvent(broadcastId, event);
    });

    // Wire output events to LogBroadcaster
    runner.on("output", (entry) => {
      // Broadcast the log entry to clients
      broadcaster.broadcast(broadcastId, entry);

      // Also check if this line is an event and broadcast if so
      eventParser.parseLine(entry.line);
    });

    // Wire state changes to LogBroadcaster
    runner.on("stateChange", (state: RunnerState, _previousState: RunnerState) => {
      broadcaster.broadcastStatus(broadcastId, state);
    });

    // Broadcast task start
    broadcaster.broadcastStatus(broadcastId, "starting");

    try {
      // Execute the ulf command
      const result = await runner.run(payload.prompt, payload.args ?? [], context.signal);

      // Broadcast final status based on result
      broadcaster.broadcastStatus(broadcastId, result.state);

      // Clean up
      runner.dispose();

      // If the runner result indicates failure, throw to trigger Dispatcher's failure path
      // This ensures task.failed event is published instead of task.completed
      if (result.state === RunnerState.FAILED) {
        throw new Error(result.error || `Process exited with code ${result.exitCode ?? 1}`);
      }

      return result;
    } catch (error) {
      // Broadcast failure status first, then the error details
      broadcaster.broadcastStatus(broadcastId, "failed");
      const errorMsg = error instanceof Error ? error.message : String(error);
      broadcaster.broadcastError(broadcastId, errorMsg);

      // Clean up
      runner.dispose();

      throw error;
    }
  };
}
