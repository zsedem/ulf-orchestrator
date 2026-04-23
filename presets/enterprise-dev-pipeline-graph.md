# Enterprise Dev Pipeline — Visual Graph

```mermaid
flowchart TD
    subgraph LEGEND
        direction LR
        H[Hat] -->|event| H
        W((Wave Worker))
        G{{Gate / Check}}
        HU[(Human)]
    end

    subgraph PHASE1["🔄 PHASE 1: Analysis Loop"]
        direction TB
        START([task.start]) --> ANALYST["🔍 Analyst"]
        ANALYST -->|human.interact| HU1[(Human)]
        HU1 -->|human.response| ANALYST
        ANALYST -->|analysis.complete| COORD1["🎯 Coordinator"]
        COORD1 -->|analysis.requested| CRITIQUE["🧐 Critique"]
        CRITIQUE -->|analysis.insufficient| ANALYST
        CRITIQUE -->|analysis.approved| COORD1
        COORD1 -->|planning.start| PLANNER
    end

    subgraph PHASE2["🔄 PHASE 2: Planning Loop"]
        direction TB
        PLANNER["📋 Planner"] -->|plan.draft| WAVE_DISPATCH1["ralph wave emit plan.critique"]
        WAVE_DISPATCH1 --> CW1["🔎 Critique Worker<br/>code-refactor"]
        WAVE_DISPATCH1 --> CW2["🔎 Critique Worker<br/>product-vision"]
        WAVE_DISPATCH1 --> CW3["🔎 Critique Worker<br/>feature-breaks"]
        WAVE_DISPATCH1 --> CW4["🔎 Critique Worker<br/>devops"]

        CW1 -->|critique.done| SYNTH1["📝 Critique Synthesizer"]
        CW2 -->|critique.done| SYNTH1
        CW3 -->|critique.done| SYNTH1
        CW4 -->|critique.done| SYNTH1

        SYNTH1 -->|plan.critiqued| PLANNER
        PLANNER -->|plan.finalized| COORD2["🎯 Coordinator"]
        COORD2 -->|bdd.start| BDD_WRITER
    end

    subgraph PHASE3["🔄 PHASE 3: BDD / TDD Loop"]
        direction TB
        BDD_WRITER["🥒 BDD Writer"] -->|bdd.ready| COORD3["🎯 Coordinator"]
        COORD3 -->|dev.start| DEVELOPER["⚙️ Developer"]
        DEVELOPER -->|dev.done| GATES{{Completion Gates}}
        GATES -->|FAIL| DEVELOPER
        GATES -->|PASS| COORD4["🎯 Coordinator"]
        DEVELOPER -->|dev.blocked| DEVELOPER
    end

    subgraph PHASE4["🔄 PHASE 4: Review Wave"]
        direction TB
        COORD4 -->|review.start| REVIEW_COORD["📊 Review Coordinator"]
        REVIEW_COORD -->|review.requested| WAVE_DISPATCH2["ralph wave emit review.perspective"]
        WAVE_DISPATCH2 --> RW1["👁️ Review Worker<br/>coverage-report"]
        WAVE_DISPATCH2 --> RW2["👁️ Review Worker<br/>code"]
        WAVE_DISPATCH2 --> RW3["👁️ Review Worker<br/>tests"]
        WAVE_DISPATCH2 --> RW4["👁️ Review Worker<br/>features"]

        RW1 -->|review.done| SYNTH2["📝 Review Synthesizer"]
        RW2 -->|review.done| SYNTH2
        RW3 -->|review.done| SYNTH2
        RW4 -->|review.done| SYNTH2

        SYNTH2 -->|dev.rejected| COORD5["🎯 Coordinator"]
        SYNTH2 -->|bdd.rejected| COORD6["🎯 Coordinator"]
        SYNTH2 -->|review.complete| COORD7["🎯 Coordinator"]

        COORD5 -->|dev.start| DEVELOPER
        COORD6 -->|bdd.start| BDD_WRITER
        COORD7 -->|LOOP_COMPLETE| END([✅ DONE])
    end

    style START fill:#e1f5e1
    style END fill:#e1f5e1
    style HU1 fill:#fff3cd
    style GATES fill:#f8d7da
    style WAVE_DISPATCH1 fill:#d1ecf1
    style WAVE_DISPATCH2 fill:#d1ecf1
    style SYNTH1 fill:#d4edda
    style SYNTH2 fill:#d4edda
    style COORD1 fill:#ffe6cc
    style COORD2 fill:#ffe6cc
    style COORD3 fill:#ffe6cc
    style COORD4 fill:#ffe6cc
    style COORD5 fill:#ffe6cc
    style COORD6 fill:#ffe6cc
    style COORD7 fill:#ffe6cc
```

## Event Flow Table

| From | Event | To | Notes |
|------|-------|-----|-------|
| Loop | `task.start` | Analyst | Kickoff |
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
| BDD Writer | `bdd.ready` | Coordinator | Hand off to hub |
| Coordinator | `dev.start` | Developer | Route to development |
| Developer | `dev.done` | Completion Gates | Tests + coverage enforced |
| Gates | *(backpressure)* | Developer | **Loop back** if fail |
| Gates | *(pass)* | Coordinator | Return to hub |
| Coordinator | `review.start` | Review Coordinator | Route to review |
| Review Coordinator | `review.requested` | Wave Dispatch | Emit 4 review perspectives |
| Review Workers | `review.done` | Review Synthesizer | Aggregate waits for all 4 |
| Synthesizer | `dev.rejected` | Coordinator → Developer | **Loop back** code fixes |
| Synthesizer | `bdd.rejected` | Coordinator → BDD Writer | **Loop back** test fixes |
| Synthesizer | `review.complete` | Coordinator | Return to hub |
| Coordinator | `LOOP_COMPLETE` | — | Terminate loop |

## Loops Summary

| Loop | Hats Involved | Trigger |
|------|--------------|---------|
| **Analysis Loop** | Analyst ↔ Critique | `analysis.insufficient` |
| **Planning Loop** | Planner ↔ Critique Wave → Synthesizer | `plan.critiqued` |
| **TDD Loop** | Developer → Gates | Gate failure (backpressure) |
| **Review→Dev Loop** | Review Synthesizer → Coordinator → Developer | `dev.rejected` |
| **Review→BDD Loop** | Review Synthesizer → Coordinator → BDD Writer | `bdd.rejected` |

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
| `dev.rejected` | `dev.start` | Developer |
| `bdd.rejected` | `bdd.start` | BDD Writer |
| `review.complete` | `LOOP_COMPLETE` | Loop termination |
