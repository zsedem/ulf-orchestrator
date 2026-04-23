# Enterprise Dev Pipeline — Visual Graph

## Backend Legend

| Color | Backend | Hats |
|-------|---------|------|
| 🟣 Purple | **Claude Opus** | Analyst, Coordinator, Planner, Critique Synthesizer, Review Synthesizer |
| 🟡 Yellow | **Claude Sonnet** | BDD Writer (Gherkin specs) |
| 🔵 Blue | **Codex** | Critique, QA Writer (tests), Developer |
| 🟢 Green | **Qwen3 / opencode** | Critique Worker ×4, Review Coordinator, Review Worker ×4 |

```mermaid
flowchart TD
    subgraph LEGEND
        direction LR
        OPUS["🟣 Claude Opus"] --> SONNET["🟡 Claude Sonnet"]
        SONNET --> CODEX["🔵 Codex"]
        CODEX --> QWEN3["🟢 Qwen3/opencode"]
    end

    subgraph PHASE1["🔄 PHASE 1: Analysis Loop"]
        direction TB
        START([work.start]) --> ANALYST["🔍 Analyst<br/>🟣 Opus"]
        ANALYST -->|human.interact| HU1[(Human)]
        HU1 -->|human.response| ANALYST
        ANALYST -->|analysis.complete| COORD1["🎯 Coordinator<br/>🟣 Opus"]
        COORD1 -->|analysis.requested| CRITIQUE["🧐 Critique<br/>🔵 Codex"]
        CRITIQUE -->|analysis.insufficient| ANALYST
        CRITIQUE -->|analysis.approved| COORD1
        COORD1 -->|planning.start| PLANNER
    end

    subgraph PHASE2["🔄 PHASE 2: Planning Loop"]
        direction TB
        PLANNER["📋 Planner<br/>🟣 Opus"] -->|plan.draft| WAVE_DISPATCH1["ralph wave emit plan.critique"]
        WAVE_DISPATCH1 --> CW1["🔎 Critique Worker<br/>code-refactor<br/>🟢 Qwen3"]
        WAVE_DISPATCH1 --> CW2["🔎 Critique Worker<br/>product-vision<br/>🟢 Qwen3"]
        WAVE_DISPATCH1 --> CW3["🔎 Critique Worker<br/>feature-breaks<br/>🟢 Qwen3"]
        WAVE_DISPATCH1 --> CW4["🔎 Critique Worker<br/>devops<br/>🟢 Qwen3"]

        CW1 -->|critique.done| SYNTH1["📝 Critique Synthesizer<br/>🟣 Opus"]
        CW2 -->|critique.done| SYNTH1
        CW3 -->|critique.done| SYNTH1
        CW4 -->|critique.done| SYNTH1

        SYNTH1 -->|plan.critiqued| PLANNER
        PLANNER -->|plan.finalized| COORD2["🎯 Coordinator<br/>🟣 Opus"]
        COORD2 -->|bdd.start| BDD_WRITER
    end

    subgraph PHASE3["🔄 PHASE 3: BDD / TDD Loop"]
        direction TB
        BDD_WRITER["🥒 BDD Writer<br/>Gherkin specs<br/>🟡 Sonnet"] -->|bdd.specs_ready| QA_WRITER["🧪 QA Writer<br/>Test impl<br/>🔵 Codex"]
        QA_WRITER -->|bdd.ready| COORD3["🎯 Coordinator<br/>🟣 Opus"]
        COORD3 -->|dev.start| DEVELOPER["⚙️ Developer<br/>🔵 Codex"]
        DEVELOPER -->|dev.done| GATES{{Completion Gates}}
        GATES -->|FAIL| DEVELOPER
        GATES -->|PASS| COORD4["🎯 Coordinator<br/>🟣 Opus"]
        DEVELOPER -->|dev.blocked| DEVELOPER
    end

    subgraph PHASE4["🔄 PHASE 4: Review Wave"]
        direction TB
        COORD4 -->|review.start| REVIEW_COORD["📊 Review Coordinator<br/>🟢 Qwen3"]
        REVIEW_COORD -->|review.requested| WAVE_DISPATCH2["ralph wave emit review.perspective"]
        WAVE_DISPATCH2 --> RW1["👁️ Review Worker<br/>coverage-report<br/>🟢 Qwen3"]
        WAVE_DISPATCH2 --> RW2["👁️ Review Worker<br/>code<br/>🟢 Qwen3"]
        WAVE_DISPATCH2 --> RW3["👁️ Review Worker<br/>tests<br/>🟢 Qwen3"]
        WAVE_DISPATCH2 --> RW4["👁️ Review Worker<br/>features<br/>🟢 Qwen3"]

        RW1 -->|review.done| SYNTH2["📝 Review Synthesizer<br/>🟣 Opus"]
        RW2 -->|review.done| SYNTH2
        RW3 -->|review.done| SYNTH2
        RW4 -->|review.done| SYNTH2

        SYNTH2 -->|dev.rejected| DEVELOPER
        SYNTH2 -->|qa.rejected| QA_WRITER
        SYNTH2 -->|bdd.rejected| BDD_WRITER
        SYNTH2 -->|review.complete| COORD5["🎯 Coordinator<br/>🟣 Opus"]

        COORD5 -->|LOOP_COMPLETE| END([✅ DONE])
    end

    style START fill:#e1f5e1
    style END fill:#e1f5e1
    style HU1 fill:#fff3cd
    style GATES fill:#f8d7da
    style WAVE_DISPATCH1 fill:#d1ecf1
    style WAVE_DISPATCH2 fill:#d1ecf1
    style SYNTH1 fill:#d4edda
    style SYNTH2 fill:#d4edda

    style ANALYST fill:#e6e6fa
    style COORD1 fill:#e6e6fa
    style COORD2 fill:#e6e6fa
    style COORD3 fill:#e6e6fa
    style COORD4 fill:#e6e6fa
    style COORD5 fill:#e6e6fa
    style PLANNER fill:#e6e6fa
    style SYNTH1 fill:#e6e6fa
    style SYNTH2 fill:#e6e6fa

    style BDD_WRITER fill:#fffacd

    style CRITIQUE fill:#e3f2fd
    style QA_WRITER fill:#e3f2fd
    style DEVELOPER fill:#e3f2fd

    style CW1 fill:#e8f5e9
    style CW2 fill:#e8f5e9
    style CW3 fill:#e8f5e9
    style CW4 fill:#e8f5e9
    style REVIEW_COORD fill:#e8f5e9
    style RW1 fill:#e8f5e9
    style RW2 fill:#e8f5e9
    style RW3 fill:#e8f5e9
    style RW4 fill:#e8f5e9
```

## Event Flow Table

| From | Event | To | Notes |
|------|-------|-----|-------|
| Loop | `work.start` | Analyst | Kickoff |
| Analyst | `human.interact` | Human | Blocking question |
| Human | `human.response` | Analyst | Continues analysis |
| Analyst | `analysis.complete` | Coordinator | Hand off to hub |
| Coordinator | `analysis.requested` | Critique | Route to critique |
| Critique | `analysis.insufficient` | Analyst | **Loop back** for more evidence |
| Critique | `analysis.approved` | Coordinator | Return to hub |
| Coordinator | `planning.start` | Planner | Route to planning |
| Planner | `plan.draft` | Wave Dispatch | Emit 4 critique perspectives |
| Critique Workers | `critique.done` | Critique Synthesizer | Aggregate waits for all 4 |
| Synthesizer | `plan.critiqued` | Planner | **Loop back** for revision |
| Planner | `plan.finalized` | Coordinator | Hand off to hub |
| Coordinator | `bdd.start` | BDD Writer | Route to BDD |
| BDD Writer | `bdd.specs_ready` | QA Writer | Gherkin → test implementation |
| QA Writer | `bdd.ready` | Coordinator | Tests written, hand off to hub |
| Coordinator | `dev.start` | Developer | Route to development |
| Developer | `dev.done` | Completion Gates | Tests + coverage enforced |
| Gates | *(backpressure)* | Developer | **Loop back** if fail |
| Gates | *(pass)* | Coordinator | Return to hub |
| Coordinator | `review.start` | Review Coordinator | Route to review |
| Review Coordinator | `review.requested` | Wave Dispatch | Emit 4 review perspectives |
| Review Workers | `review.done` | Review Synthesizer | Aggregate waits for all 4 |
| Synthesizer | `dev.rejected` | Developer | **Loop back** code fixes |
| Synthesizer | `qa.rejected` | QA Writer | **Loop back** test fixes |
| Synthesizer | `bdd.rejected` | BDD Writer | **Loop back** feature fixes |
| Synthesizer | `review.complete` | Coordinator | Return to hub |
| Coordinator | `LOOP_COMPLETE` | — | Terminate loop |

## Backend Call Estimate

| Backend | Hats | Est. Calls/Run | Role |
|---------|------|---------------|------|
| **Claude Opus** | 5 | ~8–12 | Long-context reasoning, architecture, synthesis, routing |
| **Claude Sonnet** | 1 | ~2–4 | Creative spec writing (Gherkin features) |
| **Codex** | 3 | ~5–15 | Code review, test implementation, feature development |
| **Qwen3 / opencode** | 5 | ~10–25+ | Parallel wave workers, lightweight dispatch |

## Loops Summary

| Loop | Hats Involved | Trigger |
|------|--------------|---------|
| **Analysis Loop** | Analyst ↔ Critique | `analysis.insufficient` |
| **Planning Loop** | Planner ↔ Critique Wave → Synthesizer | `plan.critiqued` |
| **TDD Loop** | Developer → Gates | Gate failure (backpressure) |
| **Review→Dev Loop** | Review Synthesizer → Developer | `dev.rejected` |
| **Review→QA Loop** | Review Synthesizer → QA Writer | `qa.rejected` |
| **Review→BDD Loop** | Review Synthesizer → BDD Writer | `bdd.rejected` |

## Waves Summary

| Wave | Coordinator | Workers | Topic | Result |
|------|------------|---------|-------|--------|
| **Plan Critique** | Planner | 4 Critique Workers | `plan.critique` | `critique.done` → Synthesizer |
| **Quality Review** | Review Coordinator | 4 Review Workers | `review.perspective` | `review.done` → Synthesizer |

## Coordinator Routing Table

| Incoming Event | Outgoing Event | Target Hat |
|---------------|---------------|-----------|
| `analysis.complete` | `analysis.requested` | Critique |
| `analysis.approved` | `planning.start` | Planner |
| `plan.finalized` | `bdd.start` | BDD Writer |
| `bdd.ready` | `dev.start` | Developer |
| `dev.done` (gates pass) | `review.start` | Review Coordinator |
| `review.complete` | `LOOP_COMPLETE` | Loop termination |

## Rejection Routing (Direct — no Coordinator)

| Rejection Event | Target Hat | Fixes |
|----------------|-----------|-------|
| `dev.rejected` | Developer | Production code bugs, logic errors |
| `qa.rejected` | QA Writer | Test implementation gaps, missing assertions |
| `bdd.rejected` | BDD Writer | Gherkin scenario gaps, incomplete features |
