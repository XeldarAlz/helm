# Dry Run — Preview Orchestration Plan

You preview what the orchestrator WOULD do without actually executing anything. This lets the developer see the full execution plan before committing resources.

## Initialization

1. **Prerequisite check:** Verify `docs/GDD.md`, `docs/TDD.md`, and `docs/WORKFLOW.md` all exist. If any are missing, tell the user which to create first.
2. Read all three documents.
3. Read `CLAUDE.md` for constraints.
4. Check if `$ARGUMENTS` contains `--eco`. If present, use the eco routing table for all model assignments in the preview.

## Process

Analyze the WORKFLOW.md and produce an execution preview:

```
## Orchestration Dry Run

### Execution Summary
- Mode: [Standard | Eco] (eco shifts models down one tier for cheaper iteration)
- Total phases: X
- Total tasks: Y
- Estimated agent spawns: Z (including re-reviews)
- Max concurrent agents per phase: [list per phase]

### Phase-by-Phase Breakdown

#### Phase 1: [Name]
- Tasks: N
- Parallel batches: M (max 4 agents per batch)
- Agent assignments:
  | Batch | Task | Agent Type | Model | Files Produced |
  |-------|------|-----------|-------|----------------|
  | 1     | P1.T1 (M) | coder | sonnet | file1.cs, file2.cs |
  | 1     | P1.T2 (S) | coder | haiku | file3.cs |
  | 1     | P1.T3 (XL) | coder | opus | file4.cs |
  | R     | Review batch 1 | reviewer | opus | — |

  **Model selection** uses the routing table: reviewer=always opus, XL=opus, S coder/tester=haiku, else=sonnet. In eco mode, annotate with `[eco]` suffix and use the eco routing table (see `/orchestrate`).
  ...

#### Phase 2: [Name]
...

### Resource Estimate
- Coder agent invocations: X
- Tester agent invocations: Y
- Reviewer agent invocations: Z (1 per batch + re-reviews estimate)
- Unity setup agent invocations: W
- Total estimated agent invocations: TOTAL

### Risk Points
- [Tasks most likely to need re-review]
- [Potential file conflicts]
- [Critical path bottlenecks]

### Proceed?
Run `/orchestrate` to execute this plan.
```

## Rules
- Do NOT spawn any agents or write any code
- Do NOT modify any files except to display this preview
- Be realistic about re-review estimates (assume ~20% of tasks need one re-review)
- Show the developer exactly what they're committing to

$ARGUMENTS
