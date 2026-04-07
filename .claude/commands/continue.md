# Continue Orchestration Agent

You are the orchestrator continuing from an interrupted execution. You pick up exactly where things left off, wasting no effort on completed work.

## Initialization

1. Read `CLAUDE.md` for project constraints.
2. Read `docs/GDD.md` for game design context.
3. Read `docs/TDD.md` for technical architecture.
4. Read `docs/WORKFLOW.md` for the full execution plan.
5. Read `docs/PROGRESS.md` — this is your source of truth for what's done.

## Resume Process

### Step 1: Assess State via Event Journal

**Primary method — Event replay (preferred):**
If `docs/EVENTS.jsonl` exists, read it line-by-line and replay events to reconstruct state:
1. Initialize: `phase=0, tasks={}, agents={}, commits=[], status="unknown"`
2. For each event line, update the model:
   - `orchestration_started` → set start time, initialize task/phase counts
   - `task_status` → update task: `tasks[id].status = event.data.to`
   - `agent_spawned` → register agent with task, type, model
   - `agent_completed` / `agent_failed` → update agent status, record files
   - `review_verdict` → update task review state (PASS → done, FAIL → failed)
   - `phase_transition` → advance phase counter
   - `commit_created` → record commit SHA
   - `orchestration_paused` → note pause state
   - `error` / `blocker` → record for display
3. After replay, you have the **ground truth**: current phase, every task's final status, every agent's last state, all commits made.
4. Cross-reference with PROGRESS.md for display info (it may be stale — events are authoritative).
5. If PROGRESS.md is inconsistent with events, **rebuild PROGRESS.md** from event-derived state.

**Fallback method — PROGRESS.md heuristic (backward compatibility):**
If `docs/EVENTS.jsonl` does NOT exist, fall back to reading PROGRESS.md and determine:
- Which phase are we in?
- Which tasks are COMPLETE (with PASS review)?
- Which tasks are IN_PROGRESS (may need to be restarted — agents don't survive restarts)?
- Which tasks are PENDING?
- Are there any FAILED reviews that need re-attempts?
- Are there any blockers logged?

### Step 2: Recovery Plan
- Tasks marked IN_PROGRESS: Check if their output files exist and are complete. If yes, send to reviewer. If no, check `.claude/checkpoint/{agent-id}.md` for saved progress and restart the task with the checkpoint included as "## Previous Progress" in the agent prompt.
- Tasks marked FAILED: Re-attempt with the review feedback included in the agent prompt. Also check for checkpoint files that may contain useful context.
- Tasks marked PENDING: Schedule normally.

### Step 3: Report to User
Before resuming, show:
```
## Continuing Orchestration

**Last checkpoint:** Phase X, Task Y
**Completed:** N tasks
**Needs restart:** M tasks (were in-progress)
**Needs re-attempt:** K tasks (failed review)
**Remaining:** J tasks

Ready to resume?
```

### Step 4: Continue Execution
On user confirmation:
- **Re-activate orchestration marker** so the stop-prevention hook protects this run:
  ```bash
  echo '{"started":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'","phase":'$CURRENT_PHASE',"phaseName":"'"$PHASE_NAME"'"}' > .claude/orchestration-active.json
  ```
- Spawn agents for the current phase's remaining tasks
- Follow the same parallel dispatch, review gate, and phase gate process
- Continue updating PROGRESS.md

## Rules
- Do NOT re-run completed and reviewed tasks
- Do NOT skip the review step, even for restarted tasks
- Treat IN_PROGRESS tasks as potentially incomplete — verify before assuming done
- If PROGRESS.md is corrupted or missing, fall back to scanning the file system for what exists and rebuilding state

$ARGUMENTS
