# Helm

**A beautiful desktop GUI for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) — built for AI-powered game development.**

Helm wraps the Claude Code CLI in a polished native app, giving you a visual control center for the multi-agent Unity game factory pipeline. Describe a game idea, and Helm orchestrates specialized AI agents to produce a full Unity 6 project — design docs, architecture, code, tests, and scene setup.

---

## Features

| | |
|---|---|
| **Chat Sessions** | Interactive Claude Code conversations with markdown rendering |
| **Orchestration Dashboard** | Monitor and control multi-agent pipeline execution in real time |
| **Document Viewer** | Browse generated GDDs, TDDs, and workflow plans |
| **Code Browser** | Explore generated Unity project files |
| **Git Timeline** | Visual history of all pipeline commits |
| **Session History** | Resume previous conversations and pipelines |
| **i18n** | Internationalization support |
| **Dark Theme** | Sleek, developer-friendly dark UI |

## Pipeline

Helm drives a five-stage game creation pipeline:

```
/game-idea  →  /architect  →  /plan-workflow  →  /orchestrate  →  /build-game
   GDD            TDD           WORKFLOW          Multi-Agent       Full Run
```

Five specialized agents collaborate during orchestration:

- **Coder** — Pure C# implementation
- **Tester** — NUnit test creation and validation
- **Reviewer** — Code review and QA
- **Unity Setup** — Scene and prefab assembly via MCP
- **Committer** — Logical commit splitting

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | [Tauri 2](https://tauri.app/) (Rust) |
| Frontend | [Svelte 5](https://svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) |
| Styling | [Tailwind CSS 4](https://tailwindcss.com/) |
| Animations | [Motion](https://motion.dev/) |
| AI Backend | [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) |
| Target Engine | [Unity 6](https://unity.com/) |

## Quick Start

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for macOS
npm run build:macos
```

> Requires [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated.

## Project Structure

```
src/                  # Svelte frontend
  lib/
    views/            # Main application views
    components/       # Reusable UI components
    stores/           # Svelte state management
    i18n/             # Translations
    theme/            # Theming system
src-tauri/            # Rust backend (Tauri)
  src/                # Tauri commands and IPC
  sessions/           # Session persistence
.claude/agents/       # AI agent prompt templates
docs/                 # Generated pipeline documents
scripts/              # Build and utility scripts
```

## License

MIT
