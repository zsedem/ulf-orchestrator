# REST API Design for ulf-api Daemon

## Overview

The ulf-api daemon currently exposes a JSON-RPC v1 API at `/rpc/v1`. This document proposes adding a first-class REST API alongside the existing RPC endpoint, using standard HTTP verbs and resource-oriented URLs.

## Goals

1. **Resource-oriented design**: Model loops, tasks, workspaces, hats, and collections as REST resources
2. **Idiomatic HTTP**: Use proper status codes, content negotiation, and standard headers
3. **Backward compatibility**: Keep `/rpc/v1` running in parallel; REST endpoints live under `/api/v1`
4. **Consistency**: Match the semantics of the existing domain models and Node.js backend REST API where applicable
5. **Discoverability**: Provide OpenAPI/Swagger documentation endpoint

## Base URL

```
/api/v1
```

## Authentication

Reuse the existing authentication layer (`Authenticator` trait). REST endpoints accept auth via the same headers as RPC:

- `Authorization: Bearer <token>` (contract mode)
- `X-API-Key: <key>` (api-key mode)
- `X-Workspace-ID: <id>` header to scope requests to a workspace (when not in URL path)

## Common Response Format

All REST responses use camelCase JSON with consistent envelope structure:

### Success Response (200-299)
```json
{
  "data": { ... },
  "meta": {
    "servedBy": "ulf-api-001",
    "servedAt": "2026-04-27T10:30:00Z"
  }
}
```

For list endpoints:
```json
{
  "data": [ ... ],
  "meta": {
    "servedBy": "ulf-api-001",
    "servedAt": "2026-04-27T10:30:00Z",
    "total": 42,
    "page": 1,
    "perPage": 20
  }
}
```

### Error Response (4xx-5xx)
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Task 'task-123' not found",
    "retryable": false,
    "details": { ... }
  },
  "meta": {
    "servedBy": "ulf-api-001",
    "servedAt": "2026-04-27T10:30:00Z"
  }
}
```

HTTP Status Code Mapping (reuses existing `status_for_code`):
- `400` - Invalid request, invalid params, config invalid
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not found (resource or method)
- `409` - Conflict, idempotency conflict
- `412` - Precondition failed
- `429` - Rate limited
- `503` - Service unavailable, backpressure dropped

## Endpoint Definitions

### System & Health

#### `GET /health`
Health check (already exists, unchanged)

**Response:**
```json
{
  "status": "ok",
  "version": "v1",
  "workspaces": 3,
  "uptimeSeconds": 3600
}
```

#### `GET /api/v1/system/version`
Get daemon version information.

**Response:**
```json
{
  "data": {
    "apiVersion": "v1",
    "daemonVersion": "0.1.0",
    "supportedVersions": ["v1"]
  },
  "meta": { ... }
}
```

#### `GET /api/v1/system/capabilities`
List all supported capabilities and methods.

**Response:**
```json
{
  "data": {
    "methods": ["task.list", "task.create", ...],
    "restEndpoints": ["GET /api/v1/tasks", "POST /api/v1/tasks", ...],
    "streamTopics": ["system.heartbeat", "task.status.changed", ...],
    "authModes": ["contract", "api-key", "none"]
  },
  "meta": { ... }
}
```

---

### Workspaces

Workspaces are top-level resources that contain all other resources.

#### `GET /api/v1/workspaces`
List all workspaces.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `status` | string | Filter by status (active, error, setup) |
| `includeHealth` | boolean | Include health check results |

**Response:**
```json
{
  "data": [
    {
      "id": "main",
      "name": "Main Workspace",
      "root": "/home/user/project",
      "status": "active",
      "createdAt": "2026-04-01T00:00:00Z",
      "health": {
        "healthy": true,
        "lastCheck": "2026-04-27T10:00:00Z"
      }
    }
  ],
  "meta": { "total": 5 }
}
```

#### `POST /api/v1/workspaces`
Create a new workspace.

**Request Body:**
```json
{
  "id": "feature-branch",
  "name": "Feature Branch Workspace",
  "root": "/home/user/project-branches/feature",
  "copyFrom": "main"
}
```

**Response (201):**
```json
{
  "data": {
    "id": "feature-branch",
    "name": "Feature Branch Workspace",
    "root": "/home/user/project-branches/feature",
    "status": "setup",
    "createdAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:id`
Get workspace details.

**Response:**
```json
{
  "data": {
    "id": "main",
    "name": "Main Workspace",
    "root": "/home/user/project",
    "status": "active",
    "createdAt": "2026-04-01T00:00:00Z",
    "updatedAt": "2026-04-27T10:00:00Z"
  },
  "meta": { ... }
}
```

#### `DELETE /api/v1/workspaces/:id`
Delete a workspace and all its resources.

**Response (204):** No body

#### `GET /api/v1/workspaces/:id/status`
Get workspace operational status.

**Response:**
```json
{
  "data": {
    "status": "active",
    "runningLoops": 2,
    "pendingTasks": 5,
    "lastActivityAt": "2026-04-27T10:25:00Z"
  },
  "meta": { ... }
}
```

---

### Tasks

Tasks are scoped to a workspace.

#### `GET /api/v1/workspaces/:workspaceId/tasks`
List tasks in a workspace.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `status` | string | Filter by status (open, in_progress, closed, failed) |
| `includeArchived` | boolean | Include archived tasks |
| `priority` | number | Filter by priority (1-5) |
| `page` | number | Page number (default: 1) |
| `perPage` | number | Items per page (default: 20, max: 100) |

**Response:**
```json
{
  "data": [
    {
      "id": "task-1745754000-a1b2",
      "title": "Implement REST API endpoints",
      "status": "open",
      "priority": 1,
      "blockedBy": null,
      "createdAt": "2026-04-27T09:00:00Z",
      "updatedAt": "2026-04-27T09:00:00Z",
      "completedAt": null,
      "archivedAt": null,
      "queuedTaskId": null,
      "mergeLoopPrompt": null
    }
  ],
  "meta": { "total": 15, "page": 1, "perPage": 20 }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks`
Create a new task.

**Request Body:**
```json
{
  "title": "Add task validation",
  "priority": 2,
  "blockedBy": null,
  "autoExecute": false
}
```

**Response (201):**
```json
{
  "data": {
    "id": "task-1745757600-c3d4",
    "title": "Add task validation",
    "status": "open",
    "priority": 2,
    "blockedBy": null,
    "createdAt": "2026-04-27T10:00:00Z",
    "updatedAt": "2026-04-27T10:00:00Z"
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/tasks/:id`
Get a specific task.

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "title": "Implement REST API endpoints",
    "status": "open",
    "priority": 1,
    "blockedBy": null,
    "createdAt": "2026-04-27T09:00:00Z",
    "updatedAt": "2026-04-27T09:00:00Z",
    "completedAt": null,
    "errorMessage": null
  },
  "meta": { ... }
}
```

#### `PATCH /api/v1/workspaces/:workspaceId/tasks/:id`
Update a task (partial update).

**Request Body:**
```json
{
  "status": "in_progress",
  "priority": 1
}
```

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "title": "Implement REST API endpoints",
    "status": "in_progress",
    "priority": 1,
    "updatedAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `DELETE /api/v1/workspaces/:workspaceId/tasks/:id`
Delete a task (only allowed in terminal states: `failed`, `closed`).

**Response (204):** No body

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/close`
Close a task.

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "status": "closed",
    "completedAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/run`
Run/execute a task.

**Response:**
```json
{
  "data": {
    "success": true,
    "queuedTaskId": "qt-1745757600-ef01",
    "task": { ... }
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/retry`
Retry a failed task.

**Response:**
```json
{
  "data": {
    "success": true,
    "queuedTaskId": "qt-1745757700-gh23",
    "task": { ... }
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/cancel`
Cancel a running task.

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "status": "failed",
    "errorMessage": "Cancelled by user"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/archive`
Archive a task.

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "archivedAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/:id/unarchive`
Unarchive a task.

**Response:**
```json
{
  "data": {
    "id": "task-1745754000-a1b2",
    "archivedAt": null
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/tasks/ready`
Get all unblocked (ready) tasks.

**Response:** Same format as list, filtered to ready tasks.

#### `POST /api/v1/workspaces/:workspaceId/tasks/run-all`
Run all ready tasks.

**Request Body:**
```json
{
  "preset": "code-review"
}
```

**Response:**
```json
{
  "data": {
    "queued": 5,
    "taskIds": ["task-1", "task-2", ...]
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/tasks/clear`
Clear all tasks (destructive).

**Response (204):** No body

---

### Loops

Loops represent autonomous orchestration cycles within a workspace.

#### `GET /api/v1/workspaces/:workspaceId/loops`
List all loops in a workspace.

**Response:**
```json
{
  "data": [
    {
      "id": "loop-1745754000-x1y2",
      "status": "running",
      "location": "/home/user/project/.worktrees/loop-1745754000-x1y2",
      "prompt": "Implement user authentication",
      "mergeCommit": null
    }
  ],
  "meta": { "total": 3 }
}
```

#### `GET /api/v1/workspaces/:workspaceId/loops/:id`
Get a specific loop.

**Response:**
```json
{
  "data": {
    "id": "loop-1745754000-x1y2",
    "status": "running",
    "location": "/home/user/project/.worktrees/loop-1745754000-x1y2",
    "prompt": "Implement user authentication",
    "mergeCommit": null
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/loops/:id/status`
Get loop status and runtime information.

**Response:**
```json
{
  "data": {
    "running": true,
    "intervalMs": 5000,
    "lastProcessedAt": "2026-04-27T10:29:55Z"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/process`
Process a loop iteration manually.

**Response:**
```json
{
  "data": {
    "processed": true,
    "eventsHandled": 3
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/stop`
Stop a running loop.

**Response:**
```json
{
  "data": {
    "id": "loop-1745754000-x1y2",
    "status": "stopped"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/retry`
Retry a failed loop.

**Response:**
```json
{
  "data": {
    "id": "loop-1745754000-x1y2",
    "status": "running"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/discard`
Discard a loop and clean up resources.

**Response (204):** No body

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/prune`
Prune loop history/logs.

**Response:**
```json
{
  "data": {
    "pruned": 150,
    "freedBytes": 1048576
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/loops/:id/merge`
Get merge state for a loop.

**Response:**
```json
{
  "data": {
    "enabled": true,
    "action": "merge",
    "reason": null
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/merge`
Trigger merge for a loop.

**Response:**
```json
{
  "data": {
    "success": true,
    "taskId": "task-1745758000-ij45"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/loops/:id/merge-task`
Trigger merge task creation.

**Response:** Same as above.

---

### Hats

Hats are configuration-driven personas for orchestration. They are global but can be scoped per-workspace in the future.

#### `GET /api/v1/hats`
List all available hats.

**Response:**
```json
{
  "data": [
    {
      "key": "developer",
      "name": "Developer",
      "description": "General development hat",
      "triggers": ["task.ready"],
      "publishes": ["dev.done"],
      "isActive": true
    }
  ],
  "meta": { "total": 5 }
}
```

#### `GET /api/v1/hats/:key`
Get a specific hat definition.

**Response:**
```json
{
  "data": {
    "key": "developer",
    "name": "Developer",
    "description": "General development hat",
    "triggers": ["task.ready"],
    "publishes": ["dev.done"],
    "instructions": "You are a helpful developer...",
    "isActive": true
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/hats/active`
Get the currently active hat for a workspace.

**Response:**
```json
{
  "data": {
    "key": "developer",
    "name": "Developer",
    "isActive": true
  },
  "meta": { ... }
}
```

---

### Planning Sessions

Planning sessions are scoped to a workspace.

#### `GET /api/v1/workspaces/:workspaceId/planning`
List planning sessions.

**Response:**
```json
{
  "data": [
    {
      "id": "plan-1745754000-m3n4",
      "status": "in_progress",
      "title": "API Design",
      "createdAt": "2026-04-27T09:00:00Z",
      "updatedAt": "2026-04-27T09:30:00Z"
    }
  ],
  "meta": { "total": 2 }
}
```

#### `POST /api/v1/workspaces/:workspaceId/planning`
Start a new planning session.

**Request Body:**
```json
{
  "title": "Database Migration Plan",
  "context": "Need to migrate from SQLite to PostgreSQL",
  "preset": "architecture"
}
```

**Response (201):**
```json
{
  "data": {
    "id": "plan-1745758000-o5p6",
    "status": "awaiting_response",
    "title": "Database Migration Plan",
    "createdAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/planning/:id`
Get planning session details.

**Response:**
```json
{
  "data": {
    "id": "plan-1745754000-m3n4",
    "status": "in_progress",
    "title": "API Design",
    "prompts": [...],
    "responses": [...],
    "artifacts": [...],
    "createdAt": "2026-04-27T09:00:00Z",
    "updatedAt": "2026-04-27T09:30:00Z"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/planning/:id/respond`
Respond to a planning prompt.

**Request Body:**
```json
{
  "response": "Use connection pooling with PgBouncer"
}
```

**Response:**
```json
{
  "data": {
    "id": "plan-1745754000-m3n4",
    "status": "in_progress"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/planning/:id/resume`
Resume a paused planning session.

**Response:**
```json
{
  "data": {
    "id": "plan-1745754000-m3n4",
    "status": "in_progress"
  },
  "meta": { ... }
}
```

#### `DELETE /api/v1/workspaces/:workspaceId/planning/:id`
Delete a planning session.

**Response (204):** No body

#### `GET /api/v1/workspaces/:workspaceId/planning/:id/artifacts/:artifactId`
Get a specific artifact from a planning session.

**Response:**
```json
{
  "data": {
    "id": "artifact-1",
    "type": "markdown",
    "content": "# Migration Plan\n\n1. Setup PgBouncer..."
  },
  "meta": { ... }
}
```

---

### Collections

Collections are scoped to a workspace.

#### `GET /api/v1/workspaces/:workspaceId/collections`
List collections.

**Response:**
```json
{
  "data": [
    {
      "id": "coll-1745754000-q7r8",
      "name": "User API Endpoints",
      "description": "Collection of user-related endpoints",
      "itemCount": 12,
      "createdAt": "2026-04-27T09:00:00Z"
    }
  ],
  "meta": { "total": 3 }
}
```

#### `POST /api/v1/workspaces/:workspaceId/collections`
Create a collection.

**Request Body:**
```json
{
  "name": "Authentication Flows",
  "description": "OAuth and session management",
  "items": [...]
}
```

**Response (201):**
```json
{
  "data": {
    "id": "coll-1745758000-s9t0",
    "name": "Authentication Flows",
    "description": "OAuth and session management",
    "itemCount": 0,
    "createdAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/collections/:id`
Get collection details.

**Response:**
```json
{
  "data": {
    "id": "coll-1745754000-q7r8",
    "name": "User API Endpoints",
    "description": "Collection of user-related endpoints",
    "items": [...],
    "createdAt": "2026-04-27T09:00:00Z",
    "updatedAt": "2026-04-27T09:00:00Z"
  },
  "meta": { ... }
}
```

#### `PATCH /api/v1/workspaces/:workspaceId/collections/:id`
Update a collection.

**Request Body:**
```json
{
  "name": "User & Auth Endpoints",
  "items": [...]
}
```

**Response:**
```json
{
  "data": {
    "id": "coll-1745754000-q7r8",
    "name": "User & Auth Endpoints",
    "updatedAt": "2026-04-27T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `DELETE /api/v1/workspaces/:workspaceId/collections/:id`
Delete a collection.

**Response (204):** No body

#### `POST /api/v1/workspaces/:workspaceId/collections/:id/import`
Import items into a collection.

**Request Body:**
```json
{
  "format": "json",
  "data": "..."
}
```

**Response:**
```json
{
  "data": {
    "id": "coll-1745754000-q7r8",
    "itemCount": 15,
    "imported": 3
  },
  "meta": { ... }
}
```

#### `GET /api/v1/workspaces/:workspaceId/collections/:id/export`
Export a collection.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `format` | string | Export format (json, yaml, csv) |

**Response:**
```json
{
  "data": {
    "format": "json",
    "content": "{...}"
  },
  "meta": { ... }
}
```

---

### Presets

Presets can be global (builtin, directory) or collection-based.

#### `GET /api/v1/presets`
List all available presets.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `source` | string | Filter by source (builtin, directory, collection) |

**Response:**
```json
{
  "data": [
    {
      "id": "code-review",
      "name": "Code Review",
      "source": "builtin",
      "description": "Standard code review preset"
    },
    {
      "id": "custom-1",
      "name": "My Custom Preset",
      "source": "collection",
      "collectionId": "coll-1745754000-q7r8"
    }
  ],
  "meta": { "total": 12 }
}
```

---

### Workspace Configuration

#### `GET /api/v1/workspaces/:workspaceId/config`
Get workspace configuration.

**Response:**
```json
{
  "data": {
    "hats": { ... },
    "eventLoop": { ... },
    "memories": { ... },
    "tasks": { ... }
  },
  "meta": { ... }
}
```

#### `PATCH /api/v1/workspaces/:workspaceId/config`
Update workspace configuration (partial update).

**Request Body:**
```json
{
  "eventLoop": {
    "maxIterations": 10
  }
}
```

**Response:**
```json
{
  "data": {
    "updated": true,
    "config": { ... }
  },
  "meta": { ... }
}
```

---

### Human-in-the-Loop (RObot)

#### `GET /api/v1/workspaces/:workspaceId/human/pending`
List pending human interactions.

**Response:**
```json
{
  "data": [
    {
      "id": "hi-1745754000-u1v2",
      "question": "Should I use PostgreSQL or MySQL?",
      "createdAt": "2026-04-27T09:00:00Z",
      "timeoutMs": 300000
    }
  ],
  "meta": { "total": 1 }
}
```

#### `POST /api/v1/workspaces/:workspaceId/human/ask`
Ask a question (agent-initiated).

**Request Body:**
```json
{
  "question": "What port should the API server use?",
  "timeoutMs": 300000
}
```

**Response (201):**
```json
{
  "data": {
    "id": "hi-1745758000-w3x4",
    "question": "What port should the API server use?",
    "status": "pending"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/human/respond`
Respond to a pending question.

**Request Body:**
```json
{
  "interactionId": "hi-1745754000-u1v2",
  "response": "Use port 3000"
}
```

**Response:**
```json
{
  "data": {
    "id": "hi-1745754000-u1v2",
    "status": "answered",
    "response": "Use port 3000"
  },
  "meta": { ... }
}
```

#### `POST /api/v1/workspaces/:workspaceId/human/:id/cancel`
Cancel a pending interaction.

**Response:**
```json
{
  "data": {
    "id": "hi-1745754000-u1v2",
    "status": "cancelled"
  },
  "meta": { ... }
}
```

---

### Event Streaming

Reuse the existing WebSocket endpoint at `/rpc/v1/stream` but add a REST-friendly SSE endpoint.

#### `GET /api/v1/workspaces/:workspaceId/events`
Server-Sent Events (SSE) stream for workspace events.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `topics` | string | Comma-separated topic filters |

**Response:** `text/event-stream`

```
event: task.status.changed
data: {"taskId":"task-1","status":"in_progress","workspaceId":"main"}

event: system.heartbeat
data: {"timestamp":"2026-04-27T10:30:00Z"}
```

#### `POST /api/v1/workspaces/:workspaceId/events/subscribe`
Subscribe to events via REST (for webhook-style delivery).

**Request Body:**
```json
{
  "topics": ["task.status.changed", "loop.status.changed"],
  "webhookUrl": "https://example.com/webhooks/ulf",
  "subscriptionId": "sub-001"
}
```

**Response (201):**
```json
{
  "data": {
    "subscriptionId": "sub-001",
    "topics": ["task.status.changed", "loop.status.changed"],
    "expiresAt": "2026-04-28T10:30:00Z"
  },
  "meta": { ... }
}
```

#### `DELETE /api/v1/workspaces/:workspaceId/events/subscriptions/:id`
Unsubscribe from events.

**Response (204):** No body

---

## Implementation Steps

### Phase 1: Foundation (Week 1)

1. **Create REST framework module** (`crates/ulf-api/src/rest/`)
   - `mod.rs` - Route registration
   - `handler.rs` - Common request/response handling
   - `error.rs` - REST-specific error formatting
   - `pagination.rs` - Pagination helpers

2. **Add REST router alongside existing RPC router**
   ```rust
   // In transport.rs
   pub fn router(runtime: RpcRuntime) -> Router {
       Router::new()
           .merge(rpc_router(runtime.clone()))
           .merge(rest_router(runtime))
   }
   ```

3. **Implement response envelope serialization**
   - `RestResponse<T>` struct with `data` and `meta` fields
   - `RestError` struct for error responses
   - Axum `IntoResponse` implementations

4. **Add OpenAPI documentation endpoint**
   - `GET /api/v1/openapi.json` - OpenAPI 3.1 spec
   - `GET /api/v1/docs` - Swagger UI (optional)

### Phase 2: Core Resources (Week 2)

5. **Implement workspace endpoints**
   - `GET /api/v1/workspaces`
   - `POST /api/v1/workspaces`
   - `GET /api/v1/workspaces/:id`
   - `DELETE /api/v1/workspaces/:id`
   - `GET /api/v1/workspaces/:id/status`

6. **Implement task endpoints**
   - Full CRUD + actions (run, retry, cancel, close, archive, etc.)
   - Query parameter parsing for filters
   - Pagination support

7. **Implement loop endpoints**
   - Read + action endpoints (no create; loops are spawned internally)

### Phase 3: Supporting Resources (Week 3)

8. **Implement hat endpoints**
   - `GET /api/v1/hats`
   - `GET /api/v1/hats/:key`
   - `GET /api/v1/workspaces/:id/hats/active`

9. **Implement planning session endpoints**
   - Full CRUD + respond/resume actions

10. **Implement collection endpoints**
    - Full CRUD + import/export

11. **Implement preset endpoints**
    - `GET /api/v1/presets`

12. **Implement config endpoints**
    - `GET /api/v1/workspaces/:id/config`
    - `PATCH /api/v1/workspaces/:id/config`

### Phase 4: Advanced Features (Week 4)

13. **Implement human-in-the-loop endpoints**
    - Pending questions, ask, respond, cancel

14. **Implement SSE streaming**
    - Add SSE endpoint as alternative to WebSocket
    - Topic filtering
    - Reuse existing `StreamDomain`

15. **Add webhook subscription support**
    - REST-friendly subscription management
    - Delivery retry logic

### Phase 5: Testing & Polish (Week 5)

16. **Write integration tests**
    - Follow existing test patterns in `crates/ulf-api/tests/`
    - Test each endpoint with success and error cases
    - Test auth propagation

17. **Add REST client SDK generation**
    - Generate TypeScript types from OpenAPI spec
    - Provide `ulf-rest-client` package

18. **Documentation**
    - Update API documentation
    - Add REST usage examples
    - Migration guide from RPC to REST

## File Structure

```
crates/ulf-api/src/
├── rest/
│   ├── mod.rs           # Route registration
│   ├── handler.rs       # Common REST handlers
│   ├── error.rs         # REST error formatting
│   ├── pagination.rs    # Pagination helpers
│   ├── workspace.rs     # Workspace endpoints
│   ├── task.rs          # Task endpoints
│   ├── loop.rs          # Loop endpoints
│   ├── hat.rs           # Hat endpoints
│   ├── planning.rs      # Planning session endpoints
│   ├── collection.rs    # Collection endpoints
│   ├── preset.rs        # Preset endpoints
│   ├── config.rs        # Config endpoints
│   ├── human.rs         # Human-in-the-loop endpoints
│   └── stream.rs        # SSE streaming endpoints
├── transport.rs         # Updated to mount both RPC and REST routers
└── ...
```

## Design Decisions

### Why keep RPC alongside REST?
- Backward compatibility for existing clients
- Some operations (batch requests, complex queries) are better suited to RPC
- WebSocket streaming already works well over RPC

### Why workspace-scoped resources?
- The daemon is fundamentally multi-tenant via workspaces
- Workspace isolation is a core security boundary
- Matches existing domain model (`WorkspaceRuntime`)

### Why `/api/v1/workspaces/:id/tasks` instead of `/api/v1/tasks?workspaceId=...`?
- Resource hierarchy is clearer
- Easier to enforce workspace-level auth
- More cache-friendly URL structure
- Aligns with REST best practices

### Why SSE in addition to WebSocket?
- SSE is simpler for clients (just HTTP)
- Automatic reconnection via HTTP
- Works better with proxies and load balancers
- WebSocket remains for bidirectional communication

## Migration Path

For clients currently using RPC:

| RPC Method | REST Endpoint |
|-----------|---------------|
| `task.list` | `GET /api/v1/workspaces/:id/tasks` |
| `task.create` | `POST /api/v1/workspaces/:id/tasks` |
| `task.get` | `GET /api/v1/workspaces/:id/tasks/:taskId` |
| `task.update` | `PATCH /api/v1/workspaces/:id/tasks/:taskId` |
| `task.close` | `POST /api/v1/workspaces/:id/tasks/:taskId/close` |
| `task.run` | `POST /api/v1/workspaces/:id/tasks/:taskId/run` |
| `loop.list` | `GET /api/v1/workspaces/:id/loops` |
| `loop.status` | `GET /api/v1/workspaces/:id/loops/:loopId/status` |
| `loop.stop` | `POST /api/v1/workspaces/:id/loops/:loopId/stop` |
| `config.get` | `GET /api/v1/workspaces/:id/config` |
| `config.update` | `PATCH /api/v1/workspaces/:id/config` |

## Future Enhancements

1. **GraphQL endpoint** - For complex queries that need flexible field selection
2. **gRPC support** - For high-performance internal communication
3. **API versioning strategy** - URL-based (`/api/v2/`) or header-based
4. **Rate limiting** - Per-workspace or per-client token bucket
5. **Request caching** - ETag support for immutable resources
6. **Bulk operations** - `POST /api/v1/workspaces/:id/tasks/bulk` for batch updates
