/**
 * Runner module
 *
 * Provides the UlfRunner service for spawning and managing ulf run child processes.
 * This is Step 4 of the implementation - the bridge between task execution and actual CLI invocation.
 */

// State management
export {
  RunnerState,
  isTerminalRunnerState,
  isValidRunnerTransition,
  getAllowedRunnerTransitions,
} from "./RunnerState";

// Log capture
export { LogStream } from "./LogStream";
export type { LogEntry, LogCallback, LogStreamOptions } from "./LogStream";

// Prompt management
export { PromptWriter } from "./PromptWriter";
export type { PromptContent, PromptWriterOptions } from "./PromptWriter";

// Main runner service
export { UlfRunner } from "./UlfRunner";
export type { UlfRunnerOptions, RunnerResult, UlfRunnerEvents } from "./UlfRunner";
export { createTestLogTaskHandler } from "./TestLogTaskHandler";
export type { TestLogTaskPayload } from "./TestLogTaskHandler";

// Task handler factory (integrates with Dispatcher and LogBroadcaster)
export { createUlfTaskHandler } from "./UlfTaskHandler";
export type { UlfTaskPayload, UlfTaskHandlerOptions } from "./UlfTaskHandler";

// Event parsing (detects Ulf orchestrator events from stdout)
export { UlfEventParser } from "./UlfEventParser";
export type { UlfEvent, EventCallback } from "./UlfEventParser";
