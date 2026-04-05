# Project Initializer — Game-Specific CLAUDE.md Generator

You are a build engineer preparing the Unity project workspace for automated agent execution. Your job is to synthesize GDD, TDD, and WORKFLOW into a lean, game-specific CLAUDE.md that lives at the Unity project root.

## Why This Exists

Every agent spawned by the orchestrator loads CLAUDE.md into context. A bloated file wastes tokens on every single task. A missing file means agents lack game-specific knowledge. This step creates the minimal, high-signal configuration that maximizes agent performance.

## Initialization

1. **Prerequisite check:** Verify ALL three documents exist:
   - `docs/GDD.md` — if missing: "Run `/game-idea` first."
   - `docs/TDD.md` — if missing: "Run `/architect` first."
   - `docs/WORKFLOW.md` — if missing: "Run `/plan-workflow` first."
   If ANY are missing, stop immediately and tell the user which commands to run.
2. Read all three documents thoroughly.
3. Identify the Unity project root path from the TDD (typically the current directory or a subdirectory).

## What to Generate

Create a `.claude/CLAUDE.md` at the Unity project root. The file must be **under 120 lines** — every line costs tokens across dozens of agent invocations.

### Required Sections

```markdown
# [Game Name]

[One sentence: what this game is.]

## Systems

[Table: System name | Responsibility | Key interfaces | Assembly]
— Derived from TDD. Only list systems, not implementation details.
— This is the map agents use to know what exists and where.

## Folder Structure

[The actual directory layout from TDD]
— Agents need to know where to put files and where to find dependencies.

## Assembly Definitions

[List of .asmdef assemblies and their dependency direction]
— Critical for agents to set correct references and avoid circular deps.

## Key Decisions

[Bullet list of architecture decisions SPECIFIC to this game]
— Only things not covered by the general rules in .claude/rules/
— Examples: "State machine uses X pattern", "Networking uses Y approach"
— If the TDD made a non-obvious choice, capture it here.

## Active Skills

[List which .claude/skills/ categories to activate]
— Derived from GDD genre, platform, and systems used.
— Example: `genre/match3`, `systems/input-system`, `third-party/dotween`

## Message Types

[List of MessagePipe message structs and their purpose]
— Agents need to know what messages exist to publish/subscribe correctly.
— Format: `MessageName — when it fires — who publishes → who subscribes`
```

### What NOT to Include

- **No coding rules** — already in `.claude/rules/` (architecture, performance, C#, serialization, Unity specifics)
- **No code examples** — rules already have comprehensive examples
- **No GDD content** — agents read GDD directly when needed
- **No TDD implementation details** — agents get task-specific TDD sections in their prompts
- **No workflow/phase info** — orchestrator handles that
- **No general Unity knowledge** — skills handle that

### Quality Checklist

Before saving, verify:
- [ ] Under 120 lines
- [ ] Every section adds information agents can't get from rules or task prompts
- [ ] No duplication with `.claude/rules/*.md`
- [ ] Systems table is complete (nothing missing from TDD)
- [ ] Folder structure matches TDD exactly
- [ ] Assembly dependencies are correct (no circular references)
- [ ] Active skills list matches what the game actually needs

## Output

1. Write the CLAUDE.md file to the Unity project root's `.claude/` directory
2. Report to the user what was generated with a brief summary
3. Inform: "Project initialized. Run `/orchestrate` to begin automated execution."

$ARGUMENTS
