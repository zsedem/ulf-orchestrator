//! Core orchestration loop implementation.
//!
//! This module contains the main `run_loop_impl` function that executes
//! the Ulf orchestration loop, along with supporting types and helper
//! functions for PTY execution and termination handling.

pub(crate) mod core;
pub(crate) mod recovery;
pub(crate) mod hook_payloads;
pub(crate) mod hook_mutations;
pub(crate) mod hooks;
pub(crate) mod lifecycle;
pub(crate) mod merges;
pub(crate) mod output;
pub(crate) mod planning;
pub(crate) mod telemetry;
pub(crate) mod waves;
#[cfg(test)]
pub(crate) mod tests;

pub(crate) use core::*;
pub(crate) use recovery::*;
pub(crate) use hook_payloads::*;
pub(crate) use lifecycle::*;
pub(crate) use hook_mutations::*;
pub(crate) use hooks::*;
pub(crate) use output::*;
pub(crate) use telemetry::*;
pub(crate) use planning::*;
pub(crate) use merges::*;
pub(crate) use waves::*;

pub(crate) use anyhow::{Context, Result};
pub(crate) use std::ffi::OsStr;
pub(crate) use std::fs::{self, File};
pub(crate) use std::io::{BufWriter, IsTerminal, stdin, stdout};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::{Command, Stdio};
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Duration;
pub(crate) use tracing::{debug, error, info, warn};
pub(crate) use ulf_adapters::{
    AcpExecutor, ClaudeStreamEvent, ClaudeStreamParser, CliBackend, CliExecutor,
    ConsoleStreamHandler, ContentBlock, CopilotStreamParser, JsonRpcStreamHandler,
    OutputFormat as BackendOutputFormat, PiAssistantEvent, PiContentBlock, PiStreamEvent,
    PiStreamParser, PrettyStreamHandler, PtyConfig, PtyExecutor, QuietStreamHandler, StreamHandler,
    TuiStreamHandler,
};
pub(crate) use ulf_core::diagnostics::{HookDisposition, HookRunTelemetryEntry};
pub(crate) use ulf_core::{
    CheckpointGateRunner, CompletionAction, EventLogger, EventLoop, EventParser, EventRecord,
    GateRunResult, HookEngine, HookExecutor, HookExecutorContract, HookMutationConfig, HookOnError,
    HookPayloadBuilderInput, HookPayloadContextInput, HookPhaseEvent, HookRunRequest,
    HookRunResult, HookSuspendMode, LoopCompletionHandler, LoopContext, LoopHistory, LoopRegistry,
    MergeQueue, Record, SessionRecorder, SummaryWriter, SuspendStateRecord, SuspendStateStore,
    TerminationReason, UlfConfig, UrgentSteerStore, build_checkpoint_backpressure_payload,
};
pub(crate) use ulf_proto::{Event, GuidanceTarget, HatId, RpcEvent, RpcState, RpcTaskCounts};
pub(crate) use ulf_tui::Tui;

pub(crate) use ratatui::style::{Color, Style};
pub(crate) use ratatui::text::{Line, Span};

pub(crate) use crate::display::{
    build_tui_hat_map, print_iteration_separator, print_termination, print_wave_header,
    print_wave_summary, print_wave_worker_done,
};
pub(crate) use crate::process_management;
pub(crate) use crate::rpc_stdin::{GuidanceMessage, RpcDispatcher, run_stdin_reader, run_stdout_emitter};
pub(crate) use crate::{ColorMode, Verbosity};

