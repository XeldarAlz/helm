# Architecture

How the Helm pipeline works under the hood.

## Overview

Helm is a configuration layer for [Claude Code](https://docs.anthropic.com/en/docs/claude-code). It uses Claude Code's extensibility system — commands, rules, skills, and hooks — to create a specialized Unity game development environment. No custom runtime is needed; everything runs through the standard Claude Code CLI.

```
┌──────────────────────────────────────────────────────┐
│                    Claude Code CLI                    │
│                                                      │
│  ┌─────────┐  ┌────────┐  ┌────────┐  ┌──────────┐  │
│  │Commands │  │ Rules  │  │ Skills │  │  Hooks   │  │
│  │  (16)   │  │  (5)   │  │  (36)  │  │  (11)    │  │
│  └────┬────┘  └───┬────┘  └───┬────┘  └────┬─────┘  │
│       │           │           │             │        │
│       v           v           v             v        │
│  ┌─────────────────────────────────────────────────┐ │
│  │              Agent Execution Layer               │ │
│  │  Coder · Tester · Reviewer · Setup · Committer  │ │
│  └─────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
         │                                    │
         v                                    v
   Unity 6 Project                    docs/ Outputs
   (C# code, tests,                  (GDD, TDD, WORKFLOW,
    scenes, prefabs)                  PROGRESS, ACTIVITY_LOG)
```

## How Claude Code Loads `.claude/`

When you run `claude` in a directory containing `.claude/`, Claude Code automatically loads:

| Component | Location | Loading |
|-----------|----------|---------|
| **CLAUDE.md** | `.claude/CLAUDE.md` | Always loaded. System-level instructions for all agents. |
| **Rules** | `.claude/rules/*.md` | Always loaded. Non-negotiable coding constraints. |
| **Commands** | `.claude/commands/*.md` | Registered as `/slash-commands`. User invokes them. |
| **Skills** | `.claude/skills/**/SKILL.md` | Auto-loaded when file patterns match skill globs. |
| **Hooks** | `.claude/hooks/*.sh` | Registered in `settings.json`. Run before/after tool calls. |
| **Settings** | `.claude/settings.json` | Permissions, hook registrations, tool allowlists. |

Agents (`.claude/agents/*.md`) are NOT auto-loaded. They're prompt templates read by the `/orchestrate` command when spawning sub-agents.

## Pipeline Execution Flow

```
User runs /build-game
        │
        ├─ Stage 1: /game-idea
        │   Interactive session → writes docs/GDD.md
        │
        ├─ Stage 2: /architect
        │   Reads GDD → writes docs/TDD.md
        │
        ├─ Stage 3: /plan-workflow
        │   Reads GDD + TDD → writes docs/WORKFLOW.md
        │
        ├─ Stage 4: /init-project
        │   Reads GDD + TDD → writes CLAUDE.md at Unity project root
        │
        └─ Stage 5: /orchestrate
            Reads WORKFLOW → spawns agents per phase:
            │
            ├─ Phase N:
            │   ├─ Coder agents (parallel) → write .cs files
            │   │   └─ Hooks validate every write
            │   ├─ Tester agent → write *Tests.cs files
            │   ├─ Reviewer agent → reads all changes, PASS/FAIL
            │   │   └─ FAIL → Coder fixes → re-review
            │   ├─ Unity Setup agent → scene/prefab via MCP
            │   └─ Committer agent → logical git commits
            │
            ├─ Phase N+1: (after Phase N passes review)
            │   └─ ...
            │
            └─ docs/PROGRESS.md updated after each phase
```

## Hook System

Hooks are shell scripts that run automatically before or after Claude Code tool calls.

### Registration

Hooks are registered in `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "bash .claude/hooks/block-scene-edit.sh",
        "timeout": 3000
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "command": "bash .claude/hooks/check-pure-csharp.sh",
        "timeout": 10000
      }
    ]
  }
}
```

### Execution Model

```
Agent calls Write tool
        │
        ├─ PreToolUse hooks run (receive tool input as JSON on stdin)
        │   ├─ Exit 0 → allow
        │   └─ Exit 2 → BLOCK (tool call is rejected, agent sees error)
        │
        ├─ Tool executes (if not blocked)
        │
        └─ PostToolUse hooks run (receive tool input as JSON on stdin)
            ├─ Exit 0 → allow (warnings printed to agent)
            └─ Exit 2 → BLOCK (agent must fix and retry)
```

### Hook Input Format

Hooks receive the tool call as JSON on stdin:

```json
{
  "tool_name": "Write",
  "tool_input": {
    "file_path": "/path/to/file.cs",
    "content": "..."
  }
}
```

For Edit operations, `tool_input` includes `old_string` and `new_string`.

## Skill Auto-Loading

Skills are loaded based on glob patterns defined in their YAML frontmatter:

```markdown
---
name: Character Controller
description: 2D and 3D character controller patterns
globs: ["**/Character*.cs", "**/Player*.cs", "**/Movement*.cs"]
---
```

When an agent reads or writes a file matching a skill's globs, Claude Code automatically injects that skill's content into the agent's context. Core skills have no globs — they're always active.

### Skill Categories

```
.claude/skills/
├── core/              # Always loaded (6 skills)
│   ├── serialization-safety/
│   ├── event-systems/
│   ├── scriptable-objects/
│   ├── assembly-definitions/
│   ├── unity-mcp-patterns/
│   └── object-pooling/
├── systems/           # Loaded by file pattern (10 skills)
├── gameplay/          # Loaded by file pattern (6 skills)
├── genre/             # Loaded by file pattern (8 skills)
├── third-party/       # Loaded by file pattern (5 skills)
└── platform/          # Loaded by file pattern (1 skill)
```

## Agent System

The orchestrator spawns sub-agents using prompt templates from `.claude/agents/`:

| Template | Spawned By | Purpose |
|----------|-----------|---------|
| `coder.md` | `/orchestrate` | Implements systems, models, views in C# |
| `tester.md` | `/orchestrate` | Writes NUnit tests for every logic system |
| `reviewer.md` | `/orchestrate` | Reviews code for architecture compliance |
| `unity-setup.md` | `/orchestrate` | Assembles scenes and prefabs via Unity MCP |
| `committer.md` | `/orchestrate` | Groups changes into logical git commits |

Each agent inherits all rules and skills but gets additional role-specific instructions from its template. Agents run as Claude Code sub-processes with full tool access.

### Agent Collaboration

```
Orchestrator (reads WORKFLOW.md)
    │
    ├── Spawns Coder agents (can be parallel for independent tasks)
    │       │
    │       └── Each write triggers hooks → validates architecture
    │
    ├── Spawns Tester agent (after coders finish)
    │       │
    │       └── Writes tests → hooks check naming, structure
    │
    ├── Spawns Reviewer agent
    │       │
    │       ├── Checks architecture compliance
    │       ├── Verifies Unity compilation
    │       └── Returns PASS or FAIL with severity
    │           │
    │           └── FAIL → Coder fixes → re-review loop
    │
    ├── Spawns Unity Setup agent (after review passes)
    │       │
    │       └── Uses Unity MCP tools for scenes/prefabs
    │
    └── Spawns Committer agent (after setup)
            │
            └── Groups changes into logical commits
                (infra first, then logic, then tests, docs last)
```

## Unity Architecture Pattern

All generated code follows Model-View-System (MVS):

```
┌─────────┐     observes     ┌─────────┐
│  View   │ ──────────────── │  Model  │
│ (Mono)  │                  │ (C#)    │
└────┬────┘                  └────┬────┘
     │ calls                      │ owns
     v                            v
┌─────────┐                 ┌─────────┐
│ System  │ ───────────────>│  Model  │
│ (C#)    │    mutates      │ (C#)    │
└────┬────┘                 └─────────┘
     │
     │ publishes/subscribes
     v
┌──────────┐
│MessagePipe│
└──────────┘
```

- **Models**: Pure C# classes. State + data. No Unity API.
- **Views**: MonoBehaviours. Read models, render, forward input. No logic.
- **Systems**: Plain C# classes registered in VContainer. All game logic.
- **Communication**: MessagePipe for cross-system messaging.
- **Async**: UniTask for all async operations.
- **DI**: VContainer for all dependency wiring.

## Directory Layout (Generated Unity Project)

```
Assets/
├── Scripts/
│   ├── Core/           # Shared utilities, extensions
│   │   └── Core.asmdef
│   ├── Logic/          # Pure C# game logic (Models + Systems)
│   │   └── Logic.asmdef
│   ├── View/           # MonoBehaviour views
│   │   └── View.asmdef
│   ├── Infrastructure/ # VContainer scopes, bootstrapping
│   │   └── Infrastructure.asmdef
│   └── Editor/         # Editor-only tools
│       └── Editor.asmdef
├── Tests/
│   ├── EditMode/       # NUnit logic tests
│   └── PlayMode/       # Integration tests
├── Prefabs/
├── ScriptableObjects/
├── Scenes/
└── Resources/
```

Assembly definitions enforce dependency direction at compile time:
- View → Logic, Infrastructure
- Infrastructure → Logic, Core
- Logic → Core
- Core → (nothing)
