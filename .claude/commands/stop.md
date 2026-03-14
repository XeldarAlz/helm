# Stop / Pause Orchestration

You need to gracefully pause the current orchestration.

## Process

1. Read `docs/PROGRESS.md` to understand current state.
2. Update `docs/PROGRESS.md`:
   - Set any IN_PROGRESS tasks to `PAUSED`
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

   To resume later, run `/resume`.
   ```

## Important

- Do NOT delete any work already done
- Do NOT mark incomplete tasks as complete
- The `/resume` command will pick up from this exact state

$ARGUMENTS
