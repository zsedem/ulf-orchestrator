/**
 * @ulf-web/server
 * Ulf web dashboard server
 */

// Database exports
export {
  getDatabase,
  initializeDatabase,
  closeDatabase,
  getSqliteConnection,
  schema,
} from "./db/connection";
export type { Task, NewTask, TaskLog, NewTaskLog, Setting, NewSetting } from "./db/schema";

// Repository exports
export { TaskRepository, SettingsRepository, TaskLogRepository } from "./repositories";

// Queue exports
export {
  TaskState,
  isTerminalState,
  isValidTransition,
  getAllowedTransitions,
  TaskQueueService,
  EventBus,
  Dispatcher,
} from "./queue";
export type {
  QueuedTask,
  EnqueueOptions,
  DequeueResult,
  Event,
  EventHandler,
  Subscription,
  SubscriptionOptions,
  PublishOptions,
  PublishResult,
  TaskHandler,
  TaskExecutionContext,
  TaskExecutionResult,
  DispatcherOptions,
  DispatcherEventType,
  DispatcherStats,
} from "./queue";

// Runner exports
export {
  RunnerState,
  isTerminalRunnerState,
  isValidRunnerTransition,
  getAllowedRunnerTransitions,
  LogStream,
  PromptWriter,
  UlfRunner,
} from "./runner";
export type {
  LogEntry,
  LogCallback,
  LogStreamOptions,
  PromptContent,
  PromptWriterOptions,
  UlfRunnerOptions,
  RunnerResult,
  UlfRunnerEvents,
} from "./runner";

// API exports
export { createServer, startServer, appRouter, taskRouter, createContext } from "./api";
export type { ServerOptions, AppRouter, Context } from "./api";

console.log("Ulf Web Server initialized");
