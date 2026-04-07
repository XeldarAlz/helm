# Stop / Pause Orchestration

You need to gracefully pause the current orchestration.

## Process

1. Read `docs/PROGRESS.md` to understand current state.
2. **Log pause event** — append to `docs/EVENTS.jsonl` (if it exists):
   ```bash
   echo '{"ts":"<ISO-NOW>","event":"orchestration_paused","data":{"reason":"user requested","phase":<current>}}' >> docs/EVENTS.jsonl
   ```
3. **Remove active marker** — delete the orchestration active marker so the stop-prevention hook allows normal stops:
   ```bash
   rm -f .claude/orchestration-active.json
   ```
4. Update `docs/PROGRESS.md`:
   - Set any IN_PROGRESS tasks to `paused`
   - Add a `## Paused` section with:
     - Timestamp
     - Reason (user requested stop)
     - Which agents were running
     - What needs to happen to resume
3. Inform the user:
   ```
   Orchestration paused.
   - Phase: [current phase]
   - Tasks completed: X
   - Tasks paused: Y
   - Tasks remaining: Z

   To continue later, run `/continue`.
   ```

## Important

- Do NOT delete any work already done
- Do NOT mark incomplete tasks as complete
- The `/continue` command will pick up from this exact state

$ARGUMENTS
