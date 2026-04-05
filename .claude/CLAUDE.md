# Unity Game Factory — AI Agent Automation System

This project is an AI-powered game development pipeline for Unity 6. It uses multiple coordinated Claude Code agents to take a game idea from concept to implementation.

## Pipeline Overview

1. `/game-idea` → Interactive GDD creation (Game Design Document)
2. `/architect` → Technical Design Document (TDD) with full architecture
3. `/plan-workflow` → Phased execution plan optimized for parallel agent work
4. `/init-project` → Generates lean, game-specific CLAUDE.md at Unity project root
5. `/orchestrate` → Multi-agent orchestrator that executes the plan
6. `/build-game` → Runs the full pipeline end-to-end

## Strict Technical Constraints (All Agents Must Follow)

Detailed rules are in `.claude/rules/` — agents load these automatically. Summary:

- **Pure C# logic**: Game logic in pure C# classes, NOT MonoBehaviours. MonoBehaviours are thin wrappers. (See `rules/architecture.md`)
- **Mandatory tests**: Every logic system must have NUnit/Unity Test Framework tests.
- **No tight coupling**: Systems communicate through interfaces, events, or a message bus. (See `rules/architecture.md`)
- **Zero allocation on hot paths**: No `new`, no boxing, no LINQ in Update/FixedUpdate. (See `rules/performance.md`)
- **Unity 6 + C# 9**: Records, init-only setters, pattern matching, switch expressions. (See `rules/csharp-unity.md`)
- **Serialization safety**: `[FormerlySerializedAs]` on ANY renamed serialized field. Unity null check (`== null` not `?.`). (See `rules/serialization.md`)
- **No runtime GameObject creation**: Prefabs + pools only. (See `rules/performance.md`)
- **ScriptableObjects for config**: All configuration data as SO assets. (See `rules/architecture.md`)
- **UI must use RectTransform**: Never plain Transform under Canvas.
- **Always use TextMeshPro**: Never legacy `UnityEngine.UI.Text`.
- **Editor/Runtime separation**: Guard `UnityEditor` with `#if UNITY_EDITOR`. (See `rules/unity-specifics.md`)

## Rules (`.claude/rules/`)

Detailed coding standards loaded automatically by agents:

| Rule File | Covers |
|-----------|--------|
| `architecture.md` | Pure C# separation, interfaces, SOLID, dependency direction, no god objects |
| `csharp-unity.md` | Naming (PascalCase types, _camelCase fields), sealed by default, structure ordering |
| `performance.md` | Zero-alloc hot paths, caching, pooling, physics optimization |
| `serialization.md` | FormerlySerializedAs, Unity null checks, SerializeReference, ISerializationCallbackReceiver |
| `unity-specifics.md` | Editor guards, platform defines, lifecycle order, threading, no coroutines |

## Skills (`.claude/skills/`)

Contextual Unity knowledge auto-loaded by agents based on file patterns:

| Category | Skills | Always Active |
|----------|--------|---------------|
| **Core** (6) | serialization-safety, event-systems, scriptable-objects, assembly-definitions, unity-mcp-patterns, object-pooling | Yes |
| **Systems** (10) | urp-pipeline, input-system, addressables, cinemachine, animation, audio, physics, navmesh, ui-toolkit, shader-graph | No |
| **Gameplay** (6) | character-controller, inventory-system, dialogue-system, save-system, state-machine, procedural-generation | No |
| **Genre** (8) | hyper-casual, match3, idle-clicker, endless-runner, puzzle, rpg, platformer-2d, topdown | No |
| **Third-Party** (5) | dotween, unitask, vcontainer, textmeshpro, odin-inspector | No |
| **Platform** (1) | mobile | No |

## Agent Templates

Agent prompt templates for the orchestrator are in `.claude/agents/`:
- `coder.md` — Pure C# implementation agent
- `tester.md` — Test writing and validation agent
- `reviewer.md` — Code review and QA agent
- `unity-setup.md` — Unity scene/prefab setup agent (uses Unity MCP)
- `committer.md` — Smart phase committer (splits changes into logical commits after review passes)

## Additional Commands

- `/status` — Check current pipeline progress
- `/resume` — Resume interrupted orchestration from last checkpoint
- `/stop` — Gracefully pause orchestration, preserving state
- `/dry-run` — Preview what the orchestrator would do without executing
- `/review-code` — Manual code review on specific files
- `/add-feature` — Add a feature to an existing game (updates GDD → TDD → WORKFLOW)
- `/init-project` — Generate game-specific CLAUDE.md for the Unity project (auto-run by `/build-game`)
- `/validate` — Validate a completed phase before proceeding
- `/refine-gdd` — Iterate on an existing GDD
- `/refine-tdd` — Iterate on an existing TDD
- `/catch-up` — Generate a human-readable codebase comprehension guide (`docs/CATCH_UP.md`)

All commands with prerequisites will check for required documents before running and tell you what's missing.

## Automated Hooks

### PreToolUse — BLOCKING (run before Write/Edit)

| Hook | Behavior |
|------|----------|
| `block-scene-edit` | **BLOCKS** direct editing of `.unity`, `.prefab`, `.asset` files (use Unity MCP instead) |
| `guard-editor-runtime` | **BLOCKS** `UnityEditor` namespace in runtime code without `#if UNITY_EDITOR` guard |

### PostToolUse — Validation (run after Write/Edit)

| Hook | Behavior |
|------|----------|
| `check-pure-csharp` | **BLOCKS** if `using UnityEngine` found in Logic/Core/Systems directories |
| `check-naming-conventions` | Warns about non-PascalCase types, non-standard field naming |
| `check-no-linq-hotpath` | Warns if `System.Linq` imported alongside hot path methods |
| `check-no-runtime-instantiate` | Warns about `Instantiate()`, `new GameObject()`, `Destroy()` |
| `check-test-exists` | Reminds if a logic class has no corresponding test file |
| `check-compile` | Basic syntax checks (braces, namespace, type declaration) |
| `warn-serialization` | Warns if `[SerializeField]` renamed without `[FormerlySerializedAs]` |
| `warn-filename` | Warns if C# filename doesn't match class name (breaks MonoBehaviour) |
| `update-progress` | Logs all file activity to `docs/ACTIVITY_LOG.md` |

Hooks returning exit code 2 block the agent. All others are warnings (exit 0).

## Document Outputs

All pipeline documents are saved to `docs/`:
- `docs/GDD.md` — Game Design Document
- `docs/TDD.md` — Technical Design Document
- `docs/WORKFLOW.md` — Execution plan with phases and tasks
- `docs/PROGRESS.md` — Orchestrator progress tracking
- `docs/ACTIVITY_LOG.md` — Auto-generated file activity log
