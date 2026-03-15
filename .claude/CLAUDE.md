# Unity Game Factory — AI Agent Automation System

This project is an AI-powered game development pipeline for Unity 6. It uses multiple coordinated Claude Code agents to take a game idea from concept to implementation.

## Pipeline Overview

1. `/game-idea` → Interactive GDD creation (Game Design Document)
2. `/architect` → Technical Design Document (TDD) with full architecture
3. `/plan-workflow` → Phased execution plan optimized for parallel agent work
4. `/orchestrate` → Multi-agent orchestrator that executes the plan
5. `/build-game` → Runs the full pipeline end-to-end

## Strict Technical Constraints (All Agents Must Follow)

- **Pure C# logic**: All game logic must be in pure C# classes, NOT in MonoBehaviours. MonoBehaviours are thin wrappers that delegate to pure C# systems.
- **Mandatory tests**: Every logic system must have corresponding NUnit/Unity Test Framework tests.
- **No tight coupling**: Systems communicate through interfaces, events, or a message bus. No direct references between unrelated systems.
- **Zero allocation on hot paths**: No `new`, no boxing, no string concatenation, no LINQ in Update/FixedUpdate or any per-frame code. Use object pools, `Span<T>`, stackalloc, and pre-allocated buffers.
- **No LINQ on hot paths**: LINQ is acceptable in initialization, editor tools, and cold paths only.
- **Unity 6 + C# 9**: Use records, init-only setters, pattern matching, switch expressions, target-typed new, indices/ranges where appropriate.
- **Data-oriented thinking**: Prefer struct-based data, SoA layouts, burst-compatible code where performance matters. Use Unity's Job System and Burst Compiler for heavy computation.
- **No runtime GameObject creation**: All GameObjects must be pre-created as prefabs and instantiated from pools or activated/deactivated. Scene setup happens at edit time or via controlled pooling.
- **Prefabs and ScriptableObjects**: Use ScriptableObjects for all configuration data. Prefabs for all visual elements. Scene should be fully preparable in the editor.
- **Designer-friendly**: Expose tuning parameters via ScriptableObjects and custom inspectors. Designers should never need to touch code.
- **Modular and extensible**: Each system is self-contained, easy to extend, and follows SOLID principles.
- **UI must use RectTransform**: All UI elements under a Canvas MUST use RectTransform, never plain Transform. When creating panels, views, or any UI child, always ensure they have a RectTransform component. Plain Transform under a Canvas causes broken layouts.
- **Always use TextMeshPro**: All text elements must use TextMeshPro (`TMP_Text`, `TextMeshProUGUI` for UI, `TextMeshPro` for world-space). Never use legacy `UnityEngine.UI.Text`. Always import `TMPro` namespace for text.

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
- `/validate` — Validate a completed phase before proceeding
- `/refine-gdd` — Iterate on an existing GDD
- `/refine-tdd` — Iterate on an existing TDD

All commands with prerequisites will check for required documents before running and tell you what's missing.

## Automated Hooks (PostToolUse on Write/Edit)

These hooks run automatically on every C# file write/edit:

| Hook | Behavior |
|------|----------|
| `check-pure-csharp` | **BLOCKS** if `using UnityEngine` found in Logic/Core/Systems directories |
| `check-naming-conventions` | Warns about non-PascalCase types, non-standard field naming |
| `check-no-linq-hotpath` | Warns if `System.Linq` imported alongside hot path methods |
| `check-no-runtime-instantiate` | Warns about `Instantiate()`, `new GameObject()`, `Destroy()` |
| `check-test-exists` | Reminds if a logic class has no corresponding test file |
| `check-compile` | Basic syntax checks (braces, namespace, type declaration) |
| `update-progress` | Logs all file activity to `docs/ACTIVITY_LOG.md` |

The pure C# guard hook returns exit code 2 (blocks the agent). All others are warnings (exit 0).

## Document Outputs

All pipeline documents are saved to `docs/`:
- `docs/GDD.md` — Game Design Document
- `docs/TDD.md` — Technical Design Document
- `docs/WORKFLOW.md` — Execution plan with phases and tasks
- `docs/PROGRESS.md` — Orchestrator progress tracking
- `docs/ACTIVITY_LOG.md` — Auto-generated file activity log
