# Contributing to Helm

Helm is an AI-powered game development pipeline for Unity 6, built on Claude Code. It coordinates multiple AI agents to take a game idea from concept to working code.

**Most wanted contributions:**

1. **New genre skills** -- each genre teaches Claude how to architect a specific type of game. Adding one directly improves output quality for every user building that genre.
2. **New skills** -- system, gameplay, or third-party skills expand what the pipeline can handle.
3. **New hooks** -- validation scripts that catch more mistakes before they ship.
4. **GUI improvements** -- the Tauri 2 + Svelte 5 desktop app is young and needs work.

---

## Getting Started

```bash
# Clone
git clone https://github.com/XeldarAlz/helm.git
cd helm

# Install frontend dependencies
npm install

# Run the GUI in dev mode (Svelte frontend only)
npm run dev

# Run the full Tauri app in dev mode (requires Rust toolchain)
npm run tauri dev

# Run frontend tests
npm test

# Run Rust backend tests
cd src-tauri && cargo test
```

**Prerequisites:** Node.js 18+, Rust toolchain (for Tauri), Claude Code CLI (for testing skills/hooks/commands).

To test `.claude/` configuration changes (skills, hooks, commands, agents), you need Claude Code installed. Run `claude` in the repo root -- it picks up `.claude/settings.json` automatically.

---

## Adding a New Skill

Skills live in `.claude/skills/{category}/{skill-name}/SKILL.md`. They are contextual knowledge files that Claude loads automatically when working on matching files.

**Categories:** `core`, `systems`, `gameplay`, `genre`, `third-party`, `platform`

**Time estimate:** 20-30 minutes for a system/gameplay skill, 45-60 minutes for a genre.

### Step 1: Create the directory and file

```bash
mkdir -p .claude/skills/{category}/{skill-name}
touch .claude/skills/{category}/{skill-name}/SKILL.md
```

### Step 2: Write the SKILL.md

Every skill file starts with YAML frontmatter, followed by markdown content.

```yaml
---
name: my-skill-name
description: "One-line summary -- what this skill covers and when it matters."
globs: ["**/MatchingPattern*.cs", "**/OtherPattern*.cs"]
---
```

**Frontmatter fields:**

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Kebab-case identifier, must match directory name |
| `description` | Yes | One-line summary in quotes. Be specific about what it covers. |
| `globs` | Yes* | File patterns that trigger auto-loading. Array of glob strings. |
| `alwaysApply` | No | Set to `true` for core skills that should load on every task. Omit or `false` otherwise. |

*Core skills use `alwaysApply: true` instead of `globs`.

### Step 3: Write the content

Required sections:

1. **Overview** -- what this skill is, when to use it, 2-3 sentences max.
2. **Key Patterns** -- code examples showing the correct way to implement things. This is the most important section. Use complete, compilable C# snippets following the project's architecture rules (MVS pattern, VContainer DI, MessagePipe, UniTask).
3. **Common Pitfalls** -- mistakes Claude (or developers) commonly make. "Do this, not that" format.
4. **Performance Notes** -- allocation concerns, caching strategies, hot path considerations.

Optional but valuable: integration notes with other skills, Unity version caveats, platform-specific behavior.

### Step 4: Update CLAUDE.md

Add your skill to the skills table in `.claude/CLAUDE.md`:

```markdown
| **Category** (N) | skill-a, skill-b, your-new-skill | Yes/No |
```

### Quality bar

- System/gameplay skills: 150+ lines, 2-3 code examples minimum.
- Genre skills: 300+ lines, 3-4 full system templates. See [Adding a New Genre](#adding-a-new-genre).
- All code examples must follow the project rules in `.claude/rules/` -- MVS separation, `sealed` by default, `m_` field prefix, VContainer injection, no coroutines, no singletons.

### Example: a minimal system skill

```yaml
---
name: navmesh
description: "Unity NavMesh navigation -- agent setup, path queries, off-mesh links, dynamic obstacles, NavMeshSurface baking."
globs: ["**/Nav*.cs", "**/Navigation*.cs", "**/Pathfind*.cs"]
---

# NavMesh Navigation

## Overview

Unity's NavMesh system handles AI pathfinding...

## Key Patterns

### Agent Movement System (pure C#, injected via VContainer)

~~~csharp
public sealed class NavigationSystem : IDisposable
{
    // ...
}
~~~

## Common Pitfalls

- **Off-mesh links silently fail** if...
- **NavMeshAgent.SetDestination allocates** on first call per frame...

## Performance Notes

- Cache `NavMeshPath` instances, reuse with `CalculatePath`...
```

---

## Adding a New Genre

Genre skills live in `.claude/skills/genre/{genre-name}/SKILL.md`. They are the highest-impact contribution because they directly shape how Claude architects an entire game.

### What a genre skill must include

1. **Core loop architecture** -- the fundamental game loop as a system diagram. What systems exist, how they interact, what messages they send.
2. **3-4 complete system code templates** -- real, compilable C# classes following MVS pattern. At minimum:
   - The central gameplay system (e.g., `MatchSystem` for match-3, `WaveSystem` for tower defense)
   - The player-facing model (e.g., `BoardModel`, `RunnerModel`)
   - One supporting system (e.g., `ComboSystem`, `SpawnSystem`)
   - VContainer `LifetimeScope` wiring all pieces together
3. **Integration notes** -- which other skills this genre commonly uses (e.g., "rpg" uses `state-machine`, `save-system`, `inventory-system`).
4. **Genre-specific pitfalls** -- things that go wrong in this type of game specifically.

### Frontmatter template for genres

```yaml
---
name: tower-defense
description: "Tower defense architecture -- wave spawning, path-following enemies, tower placement/targeting/upgrading, economy system, build phase vs combat phase state machine."
globs: ["**/Tower*.cs", "**/Wave*.cs", "**/Enemy*.cs", "**/Path*.cs", "**/TD*.cs"]
---
```

**Target length:** 300+ lines. Look at existing genres in `.claude/skills/genre/` for reference.

---

## Adding or Modifying Hooks

Hooks are bash scripts in `.claude/hooks/` that run before or after Claude uses the Write/Edit tools. They enforce coding standards automatically.

### Exit code conventions

| Exit Code | Meaning | Effect |
|-----------|---------|--------|
| `0` | Allow / warn | The operation proceeds. Stdout is shown as a warning to Claude. |
| `2` | Block | The operation is **rejected**. Claude must fix the issue before retrying. |

Any other exit code is treated as an error (hook malfunction), not a block.

### Hook structure

Hooks receive JSON on stdin with the tool input (including `file_path`). They parse it with `jq`:

```bash
#!/usr/bin/env bash
set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Your validation logic here...

# To block:
echo "BLOCKED: reason here" >&2
exit 2

# To warn:
echo "WARNING: suggestion here"
exit 0
```

### Registering a hook

Add your hook to `.claude/settings.json` under the appropriate section:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/your-hook.sh",
            "timeout": 5000,
            "statusMessage": "Running your check..."
          }
        ]
      }
    ]
  }
}
```

- **PreToolUse** hooks run before the tool executes. Use exit code `2` to block.
- **PostToolUse** hooks run after. They can warn but typically don't block (use exit `0`).
- `matcher` is a regex against tool names. `Edit|Write` matches both.
- `timeout` is in milliseconds.

### Testing hooks locally

```bash
# Simulate a Write tool call
echo '{"tool_name":"Write","tool_input":{"file_path":"Assets/Scripts/Player.cs","content":"..."}}' \
  | bash .claude/hooks/your-hook.sh
echo "Exit code: $?"
```

---

## Adding a New Command

Commands are markdown prompt files in `.claude/commands/`. When a user types `/command-name` in Claude Code, the corresponding `command-name.md` is loaded as the agent's system prompt.

### File location

```
.claude/commands/{command-name}.md
```

The filename (minus `.md`) becomes the slash command name.

### Structure

Commands are plain markdown. They typically include:

1. **Role definition** -- who the agent is and what it does.
2. **Process steps** -- numbered workflow the agent follows.
3. **Prerequisites** -- what documents/files must exist before this command runs.
4. **Output format** -- what the command produces (a document, code, etc.).
5. **References to agents/skills** -- commands can instruct Claude to behave like a specific agent template.

Look at existing commands in `.claude/commands/` for the established patterns. `game-idea.md` is a good example of a conversational command; `architect.md` shows a document-generation command.

### Update CLAUDE.md

Add your command to the commands list in `.claude/CLAUDE.md`.

---

## Modifying Agent Templates

Agent templates in `.claude/agents/` define how the orchestrator's sub-agents behave. Each file is a system prompt for a specific role:

| Agent | Role |
|-------|------|
| `coder.md` | Writes pure C# implementation code following MVS pattern |
| `tester.md` | Writes NUnit/Unity Test Framework tests |
| `reviewer.md` | Reviews code for rule violations, architecture issues |
| `unity-setup.md` | Sets up Unity scenes and prefabs via Unity MCP |
| `committer.md` | Splits completed work into logical commits |

### Guidelines

- Agent templates must not contain XML documentation or comments (enforced by existing rules).
- Keep agents focused on their single responsibility.
- Reference rules from `.claude/rules/` rather than duplicating them -- agents load rules automatically.
- Test changes by running `/orchestrate` or `/build-game` on a small game idea and checking agent behavior.

---

## GUI Development

The desktop app uses **Tauri 2** (Rust backend) with a **Svelte 5** + **TypeScript** frontend and **Tailwind CSS 4**.

### Frontend (`src/`)

```
src/
  App.svelte          -- root component
  app.css             -- global styles (Tailwind)
  main.ts             -- entry point
  lib/                -- components, stores, utilities
```

**Conventions:**

- Svelte 5 runes (`$state`, `$derived`, `$effect`) -- not Svelte 4 stores.
- TypeScript strict mode.
- Tailwind CSS 4 utility classes for styling.
- Components in `src/lib/`, organized by feature.

```bash
# Dev server (frontend only, hot reload)
npm run dev

# Type checking
npm run check

# Tests
npm test

# Watch mode tests
npm run test:watch
```

### Rust Backend (`src-tauri/`)

```
src-tauri/src/
  main.rs             -- Tauri app entry
  lib.rs              -- library root
  commands/           -- Tauri IPC command handlers
  models/             -- data structures
  parser/             -- document parsing
  process/            -- subprocess management
  state/              -- app state management
  watcher/            -- file system watchers
```

```bash
cd src-tauri

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Full app dev mode (frontend + backend)
cd .. && npm run tauri dev
```

Tauri commands are the bridge between frontend and backend. They live in `src-tauri/src/commands/` and are registered in `lib.rs`.

---

## Code Style

### Unity C# (skills, code examples)

Follow the rules in `.claude/rules/`. Key points:

- `sealed` by default on all classes.
- `m_` prefix for private fields, `s_` for static, `k_` for const/static readonly.
- `[SerializeField] private` -- never public fields for inspector exposure.
- MVS pattern: Model (pure C#), View (MonoBehaviour), System (plain C# with VContainer).
- No coroutines (use UniTask), no singletons (use VContainer), no LINQ in hot paths.

### TypeScript / Svelte (GUI)

- TypeScript strict mode, no `any`.
- Svelte 5 runes syntax.
- Prefer `const` over `let`.
- Name components in PascalCase, utilities in camelCase.

### Bash (hooks)

- `set -euo pipefail` at the top.
- Parse stdin JSON with `jq`.
- Exit `0` for allow/warn, `2` for block.
- Keep hooks fast (under 5 seconds).

---

## Pull Request Process

### Branch naming

```
feat/short-description     -- new feature or skill
fix/short-description      -- bug fix
docs/short-description     -- documentation only
refactor/short-description -- code restructuring
```

### Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(skills): add tower-defense genre skill
fix(hooks): handle missing file_path in block-scene-edit
docs: update CLAUDE.md skills table
refactor(gui): extract pipeline status component
```

### What to include in your PR

- **Summary** -- 1-3 bullet points on what changed and why.
- **Test plan** -- how you verified the change works. For skills: "Tested by running `/build-game` with a [genre] game idea." For hooks: "Tested with simulated tool input JSON." For GUI: "Ran `npm test`, verified in dev mode."
- **Checklist:**
  - [ ] CLAUDE.md updated (if adding skills, commands, or hooks)
  - [ ] Existing tests pass (`npm test`, `cargo test`)
  - [ ] Hook exit codes follow conventions (0=warn, 2=block)
  - [ ] Skill code examples follow `.claude/rules/` standards

---

## Reporting Issues

Use GitHub Issues with the appropriate template:

- **Bug Report** -- something is broken. Include: steps to reproduce, expected vs actual behavior, Claude Code version.
- **Feature Request** -- an idea for improvement. Include: use case, proposed solution.
- **New Skill Request** -- a skill you'd like to see added. Include: what it covers, which category, example use cases.
- **New Genre Request** -- a genre you'd like the pipeline to support. Include: genre description, core loop, reference games.

---

## Questions?

Open a discussion on GitHub or file an issue. For quick questions about project architecture, run `/catch-up` in Claude Code from the repo root -- it generates a comprehensive codebase guide.
