# Helm — Design & Architecture Plan

> A beautiful GUI bridge for Claude Code CLI.
> Command your AI agents without touching a terminal.

**Version:** 1.0 (Planning)
**Target Platform:** macOS (Windows/Linux later)
**Date:** 2026-03-14

---

## Table of Contents

1. [Vision & Goals](#1-vision--goals)
2. [Tech Stack](#2-tech-stack)
3. [Project Structure](#3-project-structure)
4. [Design System](#4-design-system)
5. [Localization Architecture](#5-localization-architecture)
6. [Animation System](#6-animation-system)
7. [Screens & Views](#7-screens--views)
8. [Component Library](#8-component-library)
9. [Rust Backend Architecture](#9-rust-backend-architecture)
10. [Claude Code Integration Layer](#10-claude-code-integration-layer)
11. [Session Management](#11-session-management)
12. [State Management](#12-state-management)
13. [Data Flow & IPC](#13-data-flow--ipc)
14. [File Watching & Live Updates](#14-file-watching--live-updates)
15. [Error Handling & Recovery](#15-error-handling--recovery)
16. [Build & Distribution](#16-build--distribution)
17. [Development Phases](#17-development-phases)
18. [Open Questions](#18-open-questions)

---

## 1. Vision & Goals

### What Helm Is
A desktop application that wraps Claude Code CLI in a polished, animated, professional GUI. Users interact with buttons, chat bubbles, progress bars, and visual panels instead of typing terminal commands. Claude Code runs as a hidden background process — Helm is the window into it.

### Target User
Non-terminal users: game designers, producers, junior devs, or anyone who finds terminal commands and slash-commands intimidating. They want to:
- Click "New Game" instead of typing `/game-idea`
- See agent progress as visual tiles instead of scrolling log text
- Know what to do next without memorizing command sequences

### Core Principles
1. **Beautiful by default** — Every pixel is intentional. Animations are smooth, transitions are polished.
2. **Clarity over cleverness** — UI tells you what's happening and what to do next. No guessing.
3. **Session-per-phase** — Fresh Claude Code session for each pipeline phase. No context compaction. Ever.
4. **Localization-ready** — Zero hardcoded user-facing strings from day one.
5. **Performance** — 60fps animations. No jank. Tauri's native performance makes this achievable.

---

## 2. Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Desktop Shell** | Tauri 2.0 | ~10MB bundle, native perf, Rust backend, cross-platform ready |
| **Frontend Framework** | Svelte 5 (Runes) | Compiled, fast, minimal runtime, great DX |
| **Styling** | TailwindCSS 4 | Utility-first, easy theming, JIT compilation |
| **Animations** | Svelte transitions + Motion One | Hardware-accelerated, spring-based, composable |
| **Localization** | svelte-i18n (with ICU MessageFormat) | Plurals, gender, interpolation, lazy-loaded locales |
| **Icons** | Lucide Icons (tree-shakeable SVG) | Clean, consistent, monoline style fits terminal aesthetic |
| **Fonts** | JetBrains Mono (code) + Inter (UI) | Industry standard pairing, excellent readability |
| **Markdown Rendering** | marked + shiki (syntax highlight) | Fast parsing, beautiful code blocks |
| **State** | Svelte stores + Tauri store plugin | Reactive frontend + persistent backend state |
| **Build** | Vite 6 | Fast HMR, optimized builds |

### Rust Crates (Backend)
| Crate | Purpose |
|-------|---------|
| `tauri` | App shell, IPC, window management |
| `tokio` | Async runtime for process management |
| `serde` / `serde_json` | Serialization for IPC |
| `notify` | Cross-platform file system watcher |
| `regex` | Claude Code output parsing |
| `chrono` | Timestamps for sessions and logs |
| `uuid` | Session and message IDs |
| `tauri-plugin-store` | Persistent key-value storage |
| `tauri-plugin-shell` | Child process spawning |

---

## 3. Project Structure

```
helm/
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs                 # Entry point
│   │   ├── lib.rs                  # Tauri setup, plugin registration
│   │   ├── commands/               # Tauri IPC command handlers
│   │   │   ├── mod.rs
│   │   │   ├── session.rs          # create_session, send_message, end_session
│   │   │   ├── pipeline.rs         # get_status, get_documents, get_assets
│   │   │   ├── process.rs          # spawn_claude, kill_claude, get_processes
│   │   │   └── settings.rs         # get_settings, update_settings
│   │   ├── process/                # Claude Code process management
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs          # ProcessManager: spawn, track, kill processes
│   │   │   ├── parser.rs           # OutputParser: extract structure from CLI text
│   │   │   └── bridge.rs           # Bridge: stdin writer, stdout reader, event emitter
│   │   ├── watcher/                # File system watching
│   │   │   ├── mod.rs
│   │   │   └── docs.rs             # Watch docs/ for PROGRESS.md, ACTIVITY_LOG.md changes
│   │   ├── state/                  # Application state
│   │   │   ├── mod.rs
│   │   │   ├── app_state.rs        # Global app state (sessions, settings, project path)
│   │   │   └── session_state.rs    # Per-session state (messages, status, metadata)
│   │   └── models/                 # Shared data types
│   │       ├── mod.rs
│   │       ├── session.rs          # Session, Message, SessionStatus
│   │       ├── pipeline.rs         # Phase, Task, AgentStatus, PipelineState
│   │       └── settings.rs         # AppSettings, AgentModelConfig
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/                      # App icons (icns for macOS)
│
├── src/                            # Svelte frontend
│   ├── app.html                    # HTML shell
│   ├── app.css                     # Global styles, TailwindCSS imports
│   ├── main.ts                     # Svelte mount + i18n init
│   ├── App.svelte                  # Root: splash → router → main layout
│   │
│   ├── lib/
│   │   ├── components/             # Reusable UI components
│   │   │   ├── common/             # Primitives
│   │   │   │   ├── Button.svelte
│   │   │   │   ├── IconButton.svelte
│   │   │   │   ├── Badge.svelte
│   │   │   │   ├── Tooltip.svelte
│   │   │   │   ├── Modal.svelte
│   │   │   │   ├── Toast.svelte
│   │   │   │   ├── Dropdown.svelte
│   │   │   │   ├── Toggle.svelte
│   │   │   │   ├── Slider.svelte
│   │   │   │   ├── TextInput.svelte
│   │   │   │   ├── TextArea.svelte
│   │   │   │   ├── ProgressBar.svelte
│   │   │   │   ├── Spinner.svelte
│   │   │   │   ├── Divider.svelte
│   │   │   │   └── ContextMenu.svelte
│   │   │   │
│   │   │   ├── layout/             # Structural components
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── TopBar.svelte
│   │   │   │   ├── MainLayout.svelte
│   │   │   │   ├── SplitPane.svelte
│   │   │   │   └── Panel.svelte
│   │   │   │
│   │   │   ├── splash/             # Splash screen
│   │   │   │   ├── SplashScreen.svelte
│   │   │   │   └── SplashLogo.svelte
│   │   │   │
│   │   │   ├── dashboard/          # Dashboard view components
│   │   │   │   ├── PipelineStepper.svelte
│   │   │   │   ├── QuickActions.svelte
│   │   │   │   ├── ProjectCard.svelte
│   │   │   │   ├── AssetCounter.svelte
│   │   │   │   └── RecentSessions.svelte
│   │   │   │
│   │   │   ├── chat/               # Chat interface components
│   │   │   │   ├── ChatView.svelte
│   │   │   │   ├── ChatMessage.svelte
│   │   │   │   ├── ChatInput.svelte
│   │   │   │   ├── AgentHeader.svelte
│   │   │   │   ├── ContextMeter.svelte
│   │   │   │   └── SuggestedActions.svelte
│   │   │   │
│   │   │   ├── orchestration/      # Orchestration monitor components
│   │   │   │   ├── AgentGrid.svelte
│   │   │   │   ├── AgentTile.svelte
│   │   │   │   ├── PhaseProgress.svelte
│   │   │   │   ├── TaskKanban.svelte
│   │   │   │   ├── TaskCard.svelte
│   │   │   │   ├── ReviewFeed.svelte
│   │   │   │   ├── LogStream.svelte
│   │   │   │   └── HookMonitor.svelte
│   │   │   │
│   │   │   ├── documents/          # Document viewer components
│   │   │   │   ├── DocViewer.svelte
│   │   │   │   ├── DocTabs.svelte
│   │   │   │   ├── MarkdownRenderer.svelte
│   │   │   │   ├── SectionNav.svelte
│   │   │   │   └── DiffView.svelte
│   │   │   │
│   │   │   ├── code/               # Code browser components
│   │   │   │   ├── FileTree.svelte
│   │   │   │   ├── CodeViewer.svelte
│   │   │   │   ├── ReviewOverlay.svelte
│   │   │   │   └── TestIndicator.svelte
│   │   │   │
│   │   │   ├── git/                # Git timeline components
│   │   │   │   ├── CommitTimeline.svelte
│   │   │   │   ├── CommitCard.svelte
│   │   │   │   └── DiffViewer.svelte
│   │   │   │
│   │   │   └── settings/           # Settings components
│   │   │       ├── SettingsPanel.svelte
│   │   │       ├── ModelSelector.svelte
│   │   │       ├── HookToggles.svelte
│   │   │       └── ProjectPaths.svelte
│   │   │
│   │   ├── views/                  # Page-level views (routed)
│   │   │   ├── DashboardView.svelte
│   │   │   ├── ChatSessionView.svelte
│   │   │   ├── OrchestrationView.svelte
│   │   │   ├── DocumentsView.svelte
│   │   │   ├── CodeBrowserView.svelte
│   │   │   ├── GitTimelineView.svelte
│   │   │   ├── SessionHistoryView.svelte
│   │   │   └── SettingsView.svelte
│   │   │
│   │   ├── stores/                 # Svelte stores (reactive state)
│   │   │   ├── session.ts          # Current session, message list
│   │   │   ├── pipeline.ts         # Pipeline state, phase progress
│   │   │   ├── agents.ts           # Agent statuses during orchestration
│   │   │   ├── documents.ts        # Loaded document contents
│   │   │   ├── settings.ts         # App settings
│   │   │   ├── toasts.ts           # Toast notification queue
│   │   │   └── ui.ts               # UI state (sidebar open, active view, modals)
│   │   │
│   │   ├── i18n/                   # Localization
│   │   │   ├── index.ts            # i18n setup, locale detection, lazy loading
│   │   │   ├── en.json             # English (default, complete)
│   │   │   └── keys.ts             # Type-safe key constants (auto-generated)
│   │   │
│   │   ├── theme/                  # Design system tokens
│   │   │   ├── colors.ts           # Color palette definitions
│   │   │   ├── typography.ts       # Font scales, weights
│   │   │   ├── spacing.ts          # Spacing scale
│   │   │   ├── shadows.ts          # Glow/shadow definitions
│   │   │   └── transitions.ts      # Shared transition configs
│   │   │
│   │   ├── animations/             # Animation presets
│   │   │   ├── index.ts            # Exports all animation functions
│   │   │   ├── spring.ts           # Spring configs (snappy, smooth, bouncy)
│   │   │   ├── fade.ts             # Fade in/out variants
│   │   │   ├── slide.ts            # Slide directions
│   │   │   ├── scale.ts            # Scale up/down
│   │   │   ├── stagger.ts          # Staggered list animations
│   │   │   └── glow.ts             # Glow pulse effects
│   │   │
│   │   └── utils/                  # Utilities
│   │       ├── ipc.ts              # Typed Tauri invoke wrappers
│   │       ├── markdown.ts         # Markdown parsing helpers
│   │       ├── format.ts           # Date, number, file size formatters
│   │       └── platform.ts         # OS detection helpers
│   │
│   └── assets/                     # Static assets bundled by Vite
│       ├── splash/                 # Splash screen artwork
│       │   └── helm-logo.svg       # Animated logo (SVG with CSS animations)
│       ├── icons/                  # UI icons (if not using Lucide)
│       └── fonts/                  # Self-hosted font files
│           ├── JetBrainsMono/
│           └── Inter/
│
├── package.json
├── svelte.config.js
├── tailwind.config.ts              # Theme extension, custom utilities
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## 4. Design System

### 4.1 Color Palette

Terminal-inspired dark theme with depth layers and accent colors.

```
Background Layers (darkest → lightest):
  --bg-deep:      #0A0E14    Splash screen, deepest background
  --bg-base:      #0D1117    Main app background
  --bg-surface:   #161B22    Cards, panels, sidebar
  --bg-elevated:  #1C2128    Hover states, dropdowns, modals
  --bg-overlay:   #2D333B    Selected items, active states

Border & Dividers:
  --border-subtle:  #21262D  Dividers, panel borders
  --border-default: #30363D  Input borders, card borders
  --border-active:  #58A6FF  Focused inputs, active tabs

Text:
  --text-primary:   #E6EDF3  Headings, primary content
  --text-secondary: #8B949E  Descriptions, metadata
  --text-tertiary:  #484F58  Placeholders, disabled
  --text-inverse:   #0D1117  Text on bright backgrounds

Accent — Cyan (Primary):
  --accent:         #58D5E0  Primary buttons, links, active indicators
  --accent-hover:   #7CE3EC  Button hover
  --accent-muted:   #58D5E020  Subtle backgrounds (20% opacity)
  --accent-glow:    #58D5E040  Glow effects (25% opacity)

Status Colors:
  --status-success:   #3FB950  Passed, complete, healthy
  --status-warning:   #D29922  Warnings, in-progress
  --status-error:     #F85149  Failed, blocked, critical
  --status-info:      #58A6FF  Informational
  --status-pending:   #8B949E  Pending, idle

Agent Colors (unique per agent type):
  --agent-coder:    #79C0FF  Blue — implementation
  --agent-tester:   #A5D6FF  Light blue — testing
  --agent-reviewer: #D2A8FF  Purple — review/QA
  --agent-unity:    #7EE787  Green — Unity setup
  --agent-commit:   #FFA657  Orange — git commits
```

### 4.2 Typography

```
UI Text:
  Font:     Inter
  Weights:  400 (body), 500 (labels), 600 (headings), 700 (emphasis)
  Sizes:    11px (caption), 13px (body), 15px (subtitle), 18px (title), 24px (heading), 32px (display)

Code & Terminal:
  Font:     JetBrains Mono
  Weights:  400 (code), 500 (emphasis), 700 (headings in logs)
  Sizes:    12px (log stream), 13px (code blocks), 14px (chat code)

Line Heights:
  Tight:    1.2  (headings, buttons)
  Normal:   1.5  (body text, descriptions)
  Relaxed:  1.7  (long-form reading, documents)
```

### 4.3 Spacing Scale

Base unit: 4px. All spacing uses this scale.

```
--space-1:   4px     Tight inner padding
--space-2:   8px     Default inner padding, icon gaps
--space-3:  12px     Input padding, small gaps
--space-4:  16px     Card padding, section gaps
--space-5:  20px     Medium section gaps
--space-6:  24px     Panel padding, large gaps
--space-8:  32px     View padding, major sections
--space-10: 40px     Page margins
--space-12: 48px     Splash screen spacing
--space-16: 64px     Hero spacing
```

### 4.4 Border Radius

```
--radius-sm:   4px     Badges, small tags
--radius-md:   8px     Buttons, inputs, cards
--radius-lg:  12px     Panels, modals, dropdowns
--radius-xl:  16px     Large cards, feature panels
--radius-full: 9999px  Pills, avatars, dots
```

### 4.5 Shadows & Glows

```
--shadow-sm:    0 1px 2px rgba(0, 0, 0, 0.3)
--shadow-md:    0 4px 12px rgba(0, 0, 0, 0.4)
--shadow-lg:    0 8px 24px rgba(0, 0, 0, 0.5)
--shadow-xl:    0 16px 48px rgba(0, 0, 0, 0.6)

--glow-accent:  0 0 12px var(--accent-glow), 0 0 4px var(--accent-glow)
--glow-success: 0 0 12px rgba(63, 185, 80, 0.25)
--glow-error:   0 0 12px rgba(248, 81, 73, 0.25)
--glow-pulse:   keyframe animation, 2s ease-in-out infinite
```

### 4.6 macOS Native Integration

```
- Custom title bar with native traffic lights (close/minimize/fullscreen)
- Title bar area is draggable
- Vibrancy/blur effects on sidebar (if supported by Tauri WebView)
- Respect system accent color where possible
- Cmd+, for settings (standard macOS shortcut)
- Cmd+N for new session
- Cmd+W to end current session
```

---

## 5. Localization Architecture

### Principle: Zero Hardcoded Strings

Every user-facing string goes through the i18n system. This includes:
- Button labels, headings, descriptions
- Status messages, tooltips
- Error messages, confirmations
- Placeholder text
- Accessibility labels (aria-label)

### Structure

```
src/lib/i18n/
├── index.ts          # Setup: init(), locale store, t() helper
├── keys.ts           # Type-safe key enum (auto-generated from en.json)
└── locales/
    └── en.json       # English — the source of truth
```

### Key Naming Convention

Hierarchical, dot-separated, matching component structure:

```json
{
  "app": {
    "name": "Helm",
    "tagline": "Command your AI agents"
  },
  "splash": {
    "loading": "Initializing...",
    "ready": "Ready"
  },
  "nav": {
    "dashboard": "Dashboard",
    "chat": "Session",
    "orchestration": "Orchestration",
    "documents": "Documents",
    "code": "Code",
    "git": "Git Timeline",
    "history": "Session History",
    "settings": "Settings"
  },
  "dashboard": {
    "welcome": "Welcome to Helm",
    "subtitle": "What would you like to build today?",
    "quickActions": {
      "newGame": "New Game",
      "newGameDesc": "Start from a game idea and build it end-to-end",
      "resume": "Resume Build",
      "resumeDesc": "Continue where you left off",
      "status": "Check Status",
      "statusDesc": "View current pipeline progress"
    },
    "pipeline": {
      "idea": "Game Idea",
      "architecture": "Architecture",
      "planning": "Planning",
      "building": "Building",
      "complete": "Complete"
    },
    "assets": {
      "scripts": "Scripts",
      "tests": "Tests",
      "prefabs": "Prefabs",
      "configs": "Configs"
    }
  },
  "chat": {
    "inputPlaceholder": "Type your response...",
    "send": "Send",
    "newSession": "New Session",
    "endSession": "End Session",
    "contextMeter": {
      "label": "Context",
      "low": "Healthy",
      "medium": "Getting full",
      "high": "Consider new session"
    }
  },
  "orchestration": {
    "title": "Orchestration Control",
    "phase": "Phase {current} of {total}",
    "agents": {
      "idle": "Idle",
      "running": "Running",
      "reviewing": "Reviewing",
      "passed": "Passed",
      "failed": "Failed"
    },
    "controls": {
      "pause": "Pause",
      "resume": "Resume",
      "stop": "Stop"
    }
  },
  "common": {
    "confirm": "Confirm",
    "cancel": "Cancel",
    "close": "Close",
    "save": "Save",
    "delete": "Delete",
    "loading": "Loading...",
    "error": "Something went wrong",
    "retry": "Retry"
  }
}
```

### Usage in Components

```svelte
<script>
  import { t } from '$lib/i18n';
</script>

<button>{$t('dashboard.quickActions.newGame')}</button>
<p>{$t('orchestration.phase', { current: 2, total: 7 })}</p>
```

### Adding a New Language Later

1. Create `locales/tr.json` (or any locale)
2. Translate all keys
3. Register in `index.ts`
4. Language switcher in settings picks it up automatically

---

## 6. Animation System

### Principles
- **Purpose-driven**: Every animation communicates something (entry, feedback, state change)
- **Fast**: 150-300ms for interactions, 400-600ms for transitions, 800-1200ms for splash
- **Hardware-accelerated**: Use `transform` and `opacity` only. Never animate `width`, `height`, `top`, `left`.
- **Interruptible**: Animations can be cancelled by user action without visual glitches
- **Reduced motion**: Respect `prefers-reduced-motion` — collapse to instant transitions

### Spring Presets

```typescript
export const springs = {
  // Quick, snappy — buttons, toggles, small elements
  snappy:  { stiffness: 0.15, damping: 0.8 },
  // Smooth, natural — panels, cards, modals
  smooth:  { stiffness: 0.1,  damping: 0.85 },
  // Bouncy — toasts, notifications, success states
  bouncy:  { stiffness: 0.2,  damping: 0.6 },
  // Gentle — page transitions, large elements
  gentle:  { stiffness: 0.08, damping: 0.9 },
  // Stiff — immediate feedback, hover states
  stiff:   { stiffness: 0.3,  damping: 0.95 },
};
```

### Animation Catalog

#### Splash Screen Sequence (total ~2.5s)
```
0ms      — Black screen
200ms    — Logo mark fades in (opacity 0→1, scale 0.8→1, gentle spring)
600ms    — Logo text types in letter-by-letter (50ms per char)
1000ms   — Tagline fades up (opacity 0→1, translateY 10→0)
1200ms   — Loading bar appears (width 0→100% over 800ms, ease-out)
2000ms   — Whole splash fades out (opacity 1→0, scale 1→1.02, 400ms)
2400ms   — Dashboard fades in (opacity 0→1, translateY 20→0, smooth spring)
```

#### Button Interactions
```
Idle:        Default state
Hover:       scale(1.02), background lightens, subtle glow appears (150ms ease)
Press:       scale(0.97), background darkens (100ms ease)
Release:     scale(1.0) with snappy spring
Focus:       Accent border ring with glow (200ms)
Disabled:    opacity 0.5, no hover/press effects
```

#### Card Hover
```
Hover:       translateY(-2px), shadow-md → shadow-lg, border brightens (200ms ease)
Press:       translateY(0), shadow-sm (100ms)
```

#### Sidebar Navigation
```
Item hover:   Background slides in from left (width 0→100%, 150ms ease)
Item active:  Left accent border appears (height 0→100%, snappy spring)
Icon:         Subtle scale(1.1) on hover (150ms)
```

#### Page Transitions
```
Exit:   opacity 1→0, translateX 0→-20px (200ms ease-in)
Enter:  opacity 0→1, translateX 20→0px (300ms smooth spring)
```

#### Toast Notifications
```
Enter:  translateY(-20)→0, opacity 0→1, scale 0.9→1 (bouncy spring)
Exit:   translateX 0→100%, opacity 1→0 (200ms ease-in)
```

#### Chat Messages
```
User message:    Slide in from right, opacity 0→1 (200ms ease)
Agent message:   Slide in from left, opacity 0→1 (200ms ease)
Typing indicator: 3 dots pulsing with 200ms stagger
```

#### Agent Tiles (Orchestration)
```
Status change:  Background color cross-fades (300ms), glow pulse once
Spawn:          Scale 0.8→1, opacity 0→1 (bouncy spring)
Complete:       Brief glow pulse (success color), checkmark scales in
Failed:         Shake animation (translateX ±4px, 3 cycles, 300ms), red glow
```

#### Progress Bars
```
Fill:     Width animates to target with smooth spring (never instant)
Complete: Brief shimmer effect (gradient sweep left→right, 400ms)
```

#### Staggered Lists (tasks, sessions, files)
```
Each item enters with 50ms delay after previous:
  opacity 0→1, translateY 8→0 (200ms ease)
```

#### Modal
```
Backdrop:   opacity 0→0.5 (200ms)
Content:    scale 0.95→1, opacity 0→1 (smooth spring)
Close:      Reverse, 150ms
```

---

## 7. Screens & Views

### 7.1 Splash Screen

**Trigger:** Every app launch.
**Duration:** ~2.5 seconds (can be skipped with click after 1s).

**Layout:**
```
┌─────────────────────────────────────────┐
│                                         │
│                                         │
│              [HELM LOGO]                │
│                                         │
│        Command your AI agents           │
│                                         │
│          ━━━━━━━━━━━━━░░░░              │
│            Initializing...              │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

**Behavior:**
1. Logo SVG animates in (see animation catalog)
2. Loading bar shows actual initialization progress:
   - Check Claude Code CLI is installed
   - Detect project directory
   - Read pipeline state (which docs exist)
   - Load settings
3. Transition to Dashboard

---

### 7.2 Dashboard

**Purpose:** Home base. Shows pipeline state and provides quick actions.

**Layout:**
```
┌──────┬──────────────────────────────────────────────┐
│      │  HELM                              ⚙️  ─ □ x │
│      ├──────────────────────────────────────────────┤
│  D   │                                              │
│  A   │  Welcome to Helm                             │
│  S   │  What would you like to build today?         │
│  H   │                                              │
│  B   │  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  O   │  │  🎮 New  │ │ ▶ Resume │ │ 📊 Check │     │
│  A   │  │   Game   │ │   Build  │ │  Status  │     │
│  R   │  └──────────┘ └──────────┘ └──────────┘     │
│  D   │                                              │
│      │  Pipeline Progress                           │
│  ──  │  ●━━━━━●━━━━━○━━━━━○━━━━━○                   │
│  📄  │  Idea   Arch   Plan   Build   Done           │
│  🗂  │                                              │
│  ⏱  │  ┌─ Assets ────────────────────────────┐     │
│  ⚙  │  │ 12 Scripts  8 Tests  4 SOs  3 Prefabs│    │
│      │  └────────────────────────────────────┘     │
│      │                                              │
│      │  Recent Sessions                             │
│      │  ┌─ #3 Architecture  ─ 2h ago ─ ended ──┐   │
│      │  ┌─ #2 Game Idea     ─ 3h ago ─ ended ──┐   │
│      │  ┌─ #1 Game Idea     ─ 5h ago ─ ended ──┐   │
│      │                                              │
└──────┴──────────────────────────────────────────────┘
```

**Sidebar Navigation (persistent across all views):**
```
Icon + Label (collapsible to icon-only):
  🏠 Dashboard
  💬 Session        (shows active session indicator dot)
  🤖 Orchestration  (only enabled during orchestration phase)
  📄 Documents      (badge: count of docs)
  📁 Code           (badge: file count)
  🔀 Git Timeline
  📜 History
  ──────────
  ⚙️ Settings
```

**Quick Action Cards:**
- Each is a large clickable card (~160x120px)
- Hover: lift + glow + icon animates
- Click: ripple effect → navigates to appropriate view
- "Resume Build" is disabled (dimmed) if no PROGRESS.md exists
- Cards stagger-animate in on dashboard load

**Pipeline Stepper:**
- Horizontal stepper with 5 nodes: Idea → Architecture → Planning → Building → Complete
- Completed nodes: filled accent circle + checkmark
- Current node: pulsing accent ring
- Future nodes: hollow grey circle
- Connecting lines: solid (complete) or dashed (pending)
- Click completed node → opens that document in Documents view

**Asset Counters:**
- 4 small stat cards in a row
- Each shows icon + count + label
- Counts animate up from 0 on load (stagger 100ms)

**Recent Sessions:**
- List of past session cards
- Shows: session number, phase name, time ago, status (ended/active)
- Click → opens in Session History view

---

### 7.3 Chat Session View

**Purpose:** Interactive conversation with Claude Code for phases like game-idea, architect, plan-workflow.

**Layout:**
```
┌──────┬──────────────────────────────────────────────┐
│      │  Game Idea Workshop           [Context: ████░] │
│      ├──────────────────────────────────────────────┤
│  S   │                                              │
│  I   │  ┌─ Agent ──────────────────────────────┐    │
│  D   │  │ Let's start designing your game!     │    │
│  E   │  │ What's the core concept?             │    │
│  B   │  └──────────────────────────────────────┘    │
│  A   │                                              │
│  R   │       ┌─ You ──────────────────────────┐     │
│      │       │ A roguelike deckbuilder with    │     │
│      │       │ real-time combat                │     │
│      │       └────────────────────────────────┘     │
│      │                                              │
│      │  ┌─ Agent ──────────────────────────────┐    │
│      │  │ Interesting! A few questions:         │    │
│      │  │                                      │    │
│      │  │ 1. Turn-based or real-time card play?│    │
│      │  │ 2. How many cards in a starting deck?│    │
│      │  │ 3. What's the meta-progression?      │    │
│      │  └──────────────────────────────────────┘    │
│      │                                              │
│      │  ┌─ Category Progress ────────────────────┐  │
│      │  │ ✅ Core Concept  🔄 Mechanics  ○ Tech  │  │
│      │  └────────────────────────────────────────┘  │
│      │                                              │
│      ├──────────────────────────────────────────────┤
│      │  [Type your response...              ] [Send]│
│      │                                              │
│      │  Suggested: [Turn-based] [Real-time] [Hybrid]│
│      └──────────────────────────────────────────────┘
```

**Components:**

**AgentHeader** (top bar within content area):
- Shows current phase name: "Game Idea Workshop" / "Architecture Studio" / "Workflow Planning"
- Context meter: visual bar showing estimated context usage (green→yellow→red)
- Buttons: "New Session" (+ icon), "End Session" (x icon)

**ChatMessage:**
- Agent messages: left-aligned, surface background, agent-color left border
- User messages: right-aligned, slightly different background
- Code blocks within messages: syntax highlighted, copy button
- Markdown rendered inline
- Timestamps on hover

**ChatInput:**
- Multi-line text area (auto-grows to max 6 lines)
- Send button (accent color, arrow icon)
- Keyboard shortcut: Cmd+Enter to send
- Disabled while agent is responding (shows typing indicator)

**SuggestedActions:**
- Row of pill buttons below input
- Context-aware suggestions (e.g., genre options when asking about genre)
- Click to auto-fill input
- These are generated heuristically from the agent's question type

**Category Progress** (for /game-idea specifically):
- Horizontal pill row showing question categories
- Checkmark for completed, spinner for current, empty for pending
- Parsed from agent output (detecting category transitions)

**ContextMeter:**
- Thin horizontal bar in header
- Green (0-50%), Yellow (50-75%), Red (75-100%)
- Tooltip shows: "~45,000 / 100,000 tokens used"
- When red: subtle pulse animation + suggestion toast: "Context getting full. Consider starting a new session."

---

### 7.4 Orchestration View

**Purpose:** Real-time monitoring of multi-agent execution during `/orchestrate`.

**Layout:**
```
┌──────┬──────────────────────────────────────────────┐
│      │  Orchestration Control    [Pause] [Stop]     │
│      ├──────────────────────────────────────────────┤
│      │  Phase 2 of 7: Pure C# Logic    ━━━━━━━░░░  │
│      │  Tasks: 5/12 complete  |  2 running  |  1 review │
│  S   ├──────────────────────────────────────────────┤
│  I   │                                              │
│  D   │  Agent Grid                                  │
│  E   │  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  B   │  │ 🔵 Coder │ │ 🔵 Coder │ │ 🟣 Review│     │
│  A   │  │ Agent #1 │ │ Agent #2 │ │ Agent    │     │
│  R   │  │ P2.T3    │ │ P2.T4    │ │ P2.T1    │     │
│      │  │ Running  │ │ Running  │ │ Reviewing│     │
│      │  │ ━━━━░░░  │ │ ━━░░░░░ │ │ ━━━━━━━░ │     │
│      │  └──────────┘ └──────────┘ └──────────┘     │
│      │                                              │
│      │  Task Board                                  │
│      │  Pending    │ Working     │ Review   │ Done  │
│      │  ┌───────┐  │ ┌───────┐  │┌───────┐ │ ✅T1 │
│      │  │ P2.T5 │  │ │ P2.T3 │  ││ P2.T1 │ │ ✅T2 │
│      │  │ P2.T6 │  │ │ P2.T4 │  │└───────┘ │      │
│      │  └───────┘  │ └───────┘  │          │      │
│      ├──────────────────────────────────────────────┤
│      │  Live Log                          [filter ▼]│
│      │  14:23:01 [Coder#1] Implementing InputSystem │
│      │  14:23:05 [Coder#2] Writing PoolManager      │
│      │  14:23:12 [Review]  P2.T1: PASS ✅           │
│      │  14:23:14 ⚠ Hook: naming warning in Pool.cs  │
│      └──────────────────────────────────────────────┘
```

**Components:**

**Phase Progress Bar:**
- Shows current phase number, name, and visual progress
- Below: task count breakdown (complete, running, review, pending, failed)
- Animates smoothly as tasks complete

**Agent Grid:**
- Responsive grid of agent tiles (2-4 columns depending on count)
- Each tile shows:
  - Agent type icon + color (coder=blue, tester=light blue, reviewer=purple, unity=green, committer=orange)
  - Agent ID (#1, #2, etc.)
  - Current task ID (P2.T3)
  - Status badge (Running/Reviewing/Idle/Passed/Failed)
  - Mini progress bar
- Status change triggers: background color cross-fade + glow pulse
- Failed tile: shake animation + red glow
- Completed tile: checkmark scales in + green glow pulse
- Click tile → expands to show full agent output log

**Task Kanban Board:**
- 4 columns: Pending → Working → Review → Done
- Task cards are small, showing: task ID, title, agent type badge, complexity badge (S/M/L)
- Cards animate between columns when status changes (slide transition)
- Click card → modal with full task details (description, acceptance criteria, output files)
- Done column shows green checkmarks

**Live Log Stream:**
- Auto-scrolling log with color-coded entries
- Agent messages prefixed with agent type + color
- Hook warnings in yellow
- Errors in red
- Filter dropdown: All / Agents / Hooks / Errors
- Monospace font (JetBrains Mono)
- Click log entry → jumps to relevant agent tile or file

**Hook Monitor** (collapsed by default, expandable):
- Row of small colored dots, one per hook
- Green = passed, Yellow = warning, Red = blocked
- Hover dot → tooltip with hook name and last result
- Click → expands to show recent hook output

**Control Buttons:**
- Pause: Yellow, pauses orchestration (sends `/stop`)
- Stop: Red, requires confirmation modal
- Resume: Green (visible only when paused)

---

### 7.5 Documents View

**Purpose:** Read and browse pipeline documents (GDD, TDD, WORKFLOW, PROGRESS).

**Layout:**
```
┌──────┬──────────────────────────────────────────────┐
│      │  Documents                                   │
│      ├────────┬─────────────────────────────────────┤
│  S   │ [GDD]  │  # Game Design Document v1.0       │
│  I   │ [TDD]  │                                     │
│  D   │ [WORK] │  ## 1. Executive Summary             │
│  E   │ [PROG] │  A roguelike deckbuilder with...     │
│  B   │        │                                     │
│  A   │ ────── │  ## 2. Core Concept                  │
│  R   │ Sections│  ...                                │
│      │ 1. Exec│                                     │
│      │ 2. Core│  ## 3. Game Mechanics                │
│      │ 3. Mech│  ### 3.1 Card System                 │
│      │ 4. Sys │  Cards are played from a hand of 5..│
│      │ 5. UI  │                                     │
│      │ ...    │  ```csharp                           │
│      │        │  public interface ICardSystem {      │
│      │        │      void PlayCard(CardId id);       │
│      │        │  }                                   │
│      │        │  ```                                 │
│      │        │                                     │
│      └────────┴─────────────────────────────────────┘
```

**Components:**

**DocTabs:**
- Horizontal tabs for each existing document
- Tabs show: document name + version badge
- Missing documents show disabled/dimmed tab with "Not yet created" tooltip
- Active tab: accent underline with glow

**SectionNav** (left sub-panel):
- Auto-generated table of contents from markdown headings
- Click section → smooth scroll to it
- Current section highlighted as you scroll (scroll spy)

**MarkdownRenderer:**
- Full markdown rendering with:
  - Syntax-highlighted code blocks (shiki, matching app theme)
  - Tables with alternating row colors
  - Mermaid diagram rendering (for dependency graphs in WORKFLOW)
  - Collapsible sections for long content
- Copy button on code blocks
- Search within document (Cmd+F)

**DiffView** (toggle):
- When a document is updated, show before/after diff
- Side-by-side or inline diff view
- Green for additions, red for removals

---

### 7.6 Code Browser View

**Purpose:** Browse generated source code files with review annotations.

**Layout:**
```
┌──────┬───────────┬──────────────────────────────────┐
│      │ File Tree │  InputSystem.cs                   │
│  S   │           │                                   │
│  I   │ Assets/   │  1  namespace Game.Logic.Input     │
│  D   │  Scripts/ │  2  {                              │
│  E   │   Logic/  │  3    public class InputSystem     │
│  B   │    Input/ │  4      : IInputSystem             │
│  A   │    ► Core/│  5    {                            │
│  R   │    ► UI/  │  6      private readonly Config c; │
│      │  Tests/   │  7                                 │
│      │   ► Logic/│  8      public void Tick() ...     │
│      │           │                                   │
│      │           │  ⚠ Line 42: MAJOR — Missing null  │
│      │           │    guard on input parameter        │
│      └───────────┴──────────────────────────────────┘
```

**FileTree:** Collapsible directory tree with file-type icons. Badges: green check (has tests), yellow warning (review issues).

**CodeViewer:** Syntax highlighted with line numbers. Inline review annotations (from reviewer agent output). Click annotation → details panel.

**TestIndicator:** For each logic file, shows whether a corresponding test file exists. Click → navigates to test file.

---

### 7.7 Git Timeline View

**Purpose:** Visual commit history created by the committer agent.

**Layout:** Vertical timeline with commit cards grouped by phase. Each card shows: commit type badge (feat/test/infra), message, files changed count, timestamp. Click card → diff viewer.

---

### 7.8 Session History View

**Purpose:** Browse past session transcripts (read-only).

**Layout:** List of sessions with: session ID, phase, start/end time, message count, status. Click → read-only chat view of the transcript. Search across all sessions.

---

### 7.9 Settings View

**Purpose:** Configure Helm and the underlying Claude Code integration.

**Sections:**

| Section | Controls |
|---------|----------|
| **Project** | Project directory path, Claude Code CLI path |
| **Models** | Dropdown per agent type: opus / sonnet / haiku |
| **Parallelism** | Slider: max concurrent agents (1-8) |
| **Hooks** | Toggle each hook on/off, view hook script |
| **Appearance** | Theme (dark only for v1, placeholder for light), font size |
| **Language** | Locale dropdown (only English for v1) |
| **Keyboard** | Shortcut reference table |
| **About** | Version, credits, links |

---

## 8. Component Library

### Design Principles
- Every component accepts content through i18n keys or slots, never hardcoded text
- Every interactive component has: hover, active, focus, and disabled states
- Every component respects `prefers-reduced-motion`
- Every component exposes a `class` prop for style overrides

### Component Specifications

#### Button
```
Props:
  variant:  "primary" | "secondary" | "ghost" | "danger"
  size:     "sm" | "md" | "lg"
  icon:     Lucide icon name (optional, renders left of label)
  loading:  boolean (shows spinner, disables interaction)
  disabled: boolean
  fullWidth: boolean

States:
  idle     → default styles per variant
  hover    → scale(1.02), lighten bg, glow (150ms ease)
  active   → scale(0.97), darken bg (100ms ease)
  focus    → accent ring with glow
  disabled → opacity 0.5, cursor not-allowed
  loading  → spinner replaces icon, label stays, no pointer events
```

#### IconButton
```
Props:
  icon: Lucide icon name
  size: "sm" | "md" | "lg"
  tooltip: i18n key (shown on hover)
  variant: "ghost" | "surface"

States:
  hover → bg-elevated, icon scale(1.15) (150ms)
  active → bg-overlay, icon scale(0.95) (100ms)
```

#### Badge
```
Props:
  variant: "default" | "success" | "warning" | "error" | "info" | agent color
  size: "sm" | "md"
  pulse: boolean (adds glow-pulse animation)
```

#### ProgressBar
```
Props:
  value: number (0-100)
  variant: "default" | "success" | "warning" | "error"
  size: "sm" | "md"
  animated: boolean (smooth spring to target)
  shimmer: boolean (on complete, sweep gradient)
```

#### Toast
```
Props:
  variant: "info" | "success" | "warning" | "error"
  duration: number (ms, 0 = persistent)
  action: { label: i18n key, onClick: () => void } (optional)

Behavior:
  Enter from top-right, stack vertically, auto-dismiss
  Max 3 visible, older ones pushed up
  Hover pauses auto-dismiss timer
```

#### Modal
```
Props:
  title: i18n key
  size: "sm" | "md" | "lg"
  closable: boolean
  actions: Array<{ label: i18n key, variant: ButtonVariant, onClick }>

Behavior:
  Backdrop blur + darken
  Content scales in (smooth spring)
  Close on Escape, backdrop click (if closable)
  Focus trapped within modal
```

#### Tooltip
```
Props:
  text: i18n key
  position: "top" | "bottom" | "left" | "right"
  delay: number (ms, default 500)

Behavior:
  Fade in after delay, fade out immediately on leave
  Follows component position, avoids viewport edges
```

---

## 9. Rust Backend Architecture

### 9.1 Process Manager

The core of Helm — manages Claude Code CLI processes.

```rust
pub struct ProcessManager {
    sessions: HashMap<SessionId, ClaudeProcess>,
}

pub struct ClaudeProcess {
    id: SessionId,
    child: tokio::process::Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
    status: ProcessStatus,
    started_at: DateTime<Utc>,
    working_dir: PathBuf,
}

impl ProcessManager {
    /// Spawn a new Claude Code session
    pub async fn create_session(&mut self, config: SessionConfig) -> Result<SessionId>;

    /// Send a message (user input) to a session's stdin
    pub async fn send_message(&self, session_id: &SessionId, message: &str) -> Result<()>;

    /// Send a slash command to a session
    pub async fn send_command(&self, session_id: &SessionId, command: &str) -> Result<()>;

    /// Kill a session's process
    pub async fn end_session(&mut self, session_id: &SessionId) -> Result<()>;

    /// Get status of all sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo>;

    /// Send /clear to reset context without killing process
    pub async fn clear_context(&self, session_id: &SessionId) -> Result<()>;
}
```

### 9.2 Output Parser

Parses Claude Code's stdout stream into structured events.

```rust
pub enum ClaudeEvent {
    /// Regular text output from the agent
    TextOutput { text: String },

    /// Agent is asking a question (detected by ? at end, numbered lists, etc.)
    Question { text: String, suggested_answers: Vec<String> },

    /// Agent spawned a sub-agent (detected by "Agent" tool use patterns)
    AgentSpawned { agent_type: AgentType, task: String },

    /// Agent completed
    AgentCompleted { agent_type: AgentType, result: String },

    /// File was written/edited (detected by tool use patterns)
    FileChanged { path: PathBuf, action: FileAction },

    /// Review verdict
    ReviewVerdict { task_id: String, verdict: Verdict, issues: Vec<String> },

    /// Phase transition
    PhaseChange { phase: u32, name: String },

    /// Hook output
    HookResult { hook: String, status: HookStatus, message: String },

    /// Error or warning
    Diagnostic { level: DiagLevel, message: String },

    /// Claude is still generating (for typing indicator)
    Streaming,

    /// Response complete
    ResponseComplete,
}

pub struct OutputParser {
    buffer: String,
    state: ParserState,
}

impl OutputParser {
    /// Feed raw stdout bytes and emit structured events
    pub fn parse(&mut self, chunk: &[u8]) -> Vec<ClaudeEvent>;
}
```

### 9.3 File Watcher

Watches project files for changes (especially docs/).

```rust
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
}

impl FileWatcher {
    /// Watch docs/ directory for document changes
    pub fn watch_docs(&mut self, project_dir: &Path) -> Result<()>;

    /// Watch Assets/Scripts/ for code changes
    pub fn watch_code(&mut self, project_dir: &Path) -> Result<()>;

    /// Emits FileChanged events to frontend via Tauri events
}

// Watched files and their frontend events:
// docs/PROGRESS.md  → "progress-updated"  → Orchestration view refreshes
// docs/GDD.md       → "document-updated"  → Documents view refreshes
// docs/TDD.md       → "document-updated"
// docs/WORKFLOW.md  → "document-updated"
// docs/ACTIVITY_LOG.md → "activity-logged" → Dashboard asset counts refresh
// Assets/Scripts/**/*.cs → "code-changed"  → Code browser refreshes
```

### 9.4 Session Store

Persists session transcripts and metadata.

```rust
pub struct SessionStore {
    store_dir: PathBuf,  // ~/.helm/sessions/
}

pub struct SessionRecord {
    id: SessionId,
    phase: PipelinePhase,
    messages: Vec<Message>,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    status: SessionStatus,
}

impl SessionStore {
    pub fn save_session(&self, session: &SessionRecord) -> Result<()>;
    pub fn load_session(&self, id: &SessionId) -> Result<SessionRecord>;
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>>;
    pub fn delete_session(&self, id: &SessionId) -> Result<()>;
}
```

---

## 10. Claude Code Integration Layer

### How Helm Talks to Claude Code

```
┌───────────────┐     stdin (pipe)      ┌──────────────────┐
│  Helm (Rust)  │ ───────────────────── │  claude CLI       │
│  Process Mgr  │                       │  (child process)  │
│               │ ◄──────────────────── │                  │
└───────────────┘     stdout (pipe)     └──────────────────┘
```

### Spawning a Session

```bash
# What Helm executes under the hood:
claude --verbose --output-format text
```

Flags:
- `--verbose`: More detailed output for parsing
- `--output-format text`: Consistent text output (not streaming markdown)

### Sending Messages

Write to the child process's stdin:
- User typed message → write message + newline
- Slash command → write `/game-idea\n` etc.
- Clear context → write `/clear\n`

### Reading Output

Read stdout line by line, feed to OutputParser:
- Each line is parsed for patterns (questions, agent spawns, file changes, etc.)
- Parsed events are emitted to frontend via Tauri events
- Raw text is also stored in session transcript

### Session Lifecycle

```
1. User clicks "New Game" on Dashboard
2. Helm spawns: claude --verbose --output-format text
3. Helm sends: /game-idea
4. Claude responds with questions
5. OutputParser detects Question event → frontend shows chat bubble
6. User types answer → Helm writes to stdin
7. ... (conversation continues) ...
8. Claude saves docs/GDD.md
9. FileWatcher detects GDD.md → frontend updates pipeline stepper
10. User clicks "End Session" → Helm kills process
11. SessionStore saves transcript
12. Dashboard shows "Start Architecture" button (next phase)
13. User clicks it → new claude process spawned → /architect sent
```

---

## 11. Session Management

### Session Model

```typescript
interface Session {
  id: string;              // UUID
  phase: PipelinePhase;    // "game-idea" | "architect" | "plan-workflow" | "orchestrate" | "custom"
  status: "active" | "ended" | "error";
  startedAt: string;       // ISO timestamp
  endedAt?: string;
  messageCount: number;
  contextUsage?: number;   // Estimated token count (0-1 ratio)
  transcript: Message[];
}

interface Message {
  id: string;
  role: "user" | "agent" | "system";
  content: string;         // Markdown content
  timestamp: string;
  metadata?: {
    agentType?: string;
    taskId?: string;
    event?: ClaudeEvent;
  };
}
```

### Session Rules

1. **One active session at a time** — Starting a new session ends the current one
2. **Session per phase** — Each pipeline phase (idea, arch, plan, build) gets its own session
3. **Manual sessions** — User can start a "custom" session for ad-hoc Claude Code usage
4. **Context monitoring** — If detected context usage > 75%, show warning toast
5. **Auto-archive** — Ended sessions are saved to disk and appear in History
6. **Transcript persistence** — All messages saved even if app crashes (write-ahead to disk)

### New Session Flow

```
User clicks "New Session" or phase button
  → Confirmation if active session exists: "End current session and start new?"
  → If confirmed:
    1. End current session (kill process, save transcript)
    2. Spawn new claude process
    3. If phase-specific: send slash command automatically
    4. UI transitions to Chat view
    5. Session appears in sidebar with pulsing active dot
```

### Clear Context Flow

```
User clicks "Clear Context" button in chat header
  → Confirmation: "This will clear Claude's memory of this conversation. Continue?"
  → If confirmed:
    1. Send /clear to stdin
    2. Add system message to transcript: "Context cleared"
    3. Reset context meter to 0
    4. Session continues (same process, same session ID)
```

---

## 12. State Management

### Frontend (Svelte Stores)

```typescript
// session.ts — Current session state
export const currentSession = writable<Session | null>(null);
export const messages = writable<Message[]>([]);
export const isAgentTyping = writable<boolean>(false);

// pipeline.ts — Pipeline progress
export const pipelineState = writable<PipelineState>({
  gddExists: false,
  tddExists: false,
  workflowExists: false,
  progressExists: false,
  currentPhase: "none",
});

// agents.ts — Agent states during orchestration
export const agents = writable<AgentState[]>([]);
export const tasks = writable<TaskState[]>([]);
export const currentOrchPhase = writable<number>(0);

// documents.ts — Loaded document contents
export const documents = writable<Record<string, string>>({});

// settings.ts — App settings (synced with Tauri store)
export const settings = writable<AppSettings>(defaultSettings);

// toasts.ts — Toast notification queue
export const toasts = writable<Toast[]>([]);

// ui.ts — UI state
export const activeView = writable<ViewName>("dashboard");
export const sidebarCollapsed = writable<boolean>(false);
export const activeModal = writable<ModalName | null>(null);
```

### Backend (Rust State)

```rust
pub struct AppState {
    pub process_manager: Mutex<ProcessManager>,
    pub session_store: Mutex<SessionStore>,
    pub file_watcher: Mutex<FileWatcher>,
    pub project_dir: PathBuf,
    pub settings: RwLock<AppSettings>,
}
```

### Sync Strategy

- **Settings**: Frontend ↔ Backend via Tauri store plugin (auto-persisted)
- **Session data**: Backend is source of truth, pushes events to frontend
- **Pipeline state**: Derived from filesystem (file watcher), pushed to frontend
- **UI state**: Frontend only, not persisted (except sidebar collapse pref)
- **Session history**: Backend (disk), loaded on demand

---

## 13. Data Flow & IPC

### Tauri Commands (Frontend → Backend)

```typescript
// Session management
invoke("create_session", { phase: "game-idea" }): Promise<SessionId>
invoke("send_message", { sessionId, message }): Promise<void>
invoke("send_command", { sessionId, command: "/architect" }): Promise<void>
invoke("end_session", { sessionId }): Promise<void>
invoke("clear_context", { sessionId }): Promise<void>

// Pipeline info
invoke("get_pipeline_state"): Promise<PipelineState>
invoke("read_document", { name: "GDD" }): Promise<string>
invoke("get_asset_counts"): Promise<AssetCounts>

// Settings
invoke("get_settings"): Promise<AppSettings>
invoke("update_settings", { settings }): Promise<void>

// Session history
invoke("list_sessions"): Promise<SessionSummary[]>
invoke("load_session", { sessionId }): Promise<SessionRecord>

// Code browser
invoke("get_file_tree", { path }): Promise<FileNode[]>
invoke("read_file", { path }): Promise<string>

// Git
invoke("get_git_log", { limit: 50 }): Promise<Commit[]>
invoke("get_git_diff", { commitHash }): Promise<string>
```

### Tauri Events (Backend → Frontend)

```typescript
// Claude Code output (streaming)
listen("claude-output", (event: { sessionId, text }) => ...)
listen("claude-event", (event: { sessionId, event: ClaudeEvent }) => ...)
listen("claude-complete", (event: { sessionId }) => ...)

// File system changes
listen("document-updated", (event: { name, content }) => ...)
listen("progress-updated", (event: { content }) => ...)
listen("code-changed", (event: { path, action }) => ...)
listen("activity-logged", (event: { entry }) => ...)

// Process lifecycle
listen("session-started", (event: { sessionId }) => ...)
listen("session-ended", (event: { sessionId, reason }) => ...)
listen("session-error", (event: { sessionId, error }) => ...)
```

---

## 14. File Watching & Live Updates

### Watched Paths

| Path | Event | UI Update |
|------|-------|-----------|
| `docs/GDD.md` | `document-updated` | Pipeline stepper, Documents view |
| `docs/TDD.md` | `document-updated` | Pipeline stepper, Documents view |
| `docs/WORKFLOW.md` | `document-updated` | Pipeline stepper, Documents view |
| `docs/PROGRESS.md` | `progress-updated` | Orchestration view (phase, tasks, agents) |
| `docs/ACTIVITY_LOG.md` | `activity-logged` | Dashboard asset counts |
| `Assets/Scripts/**/*.cs` | `code-changed` | Code browser, asset counts |
| `Assets/Tests/**/*.cs` | `code-changed` | Code browser, test indicators |

### Debouncing

File changes are debounced by 500ms to avoid flooding the frontend during rapid writes (e.g., agent writing multiple files).

### PROGRESS.md Parsing

The Rust backend parses PROGRESS.md on every change to extract:
- Current phase number and name
- Task statuses (PENDING, IN_PROGRESS, COMPLETE, FAILED)
- Agent assignments
- Review results

This parsed data drives the entire Orchestration view.

---

## 15. Error Handling & Recovery

### Process Crashes

If the Claude Code process exits unexpectedly:
1. Detect via process exit code
2. Show error toast with message
3. Offer "Restart Session" button
4. Session transcript is preserved (already written to disk)
5. If during orchestration: mark current tasks as interrupted, offer `/resume`

### Network Issues

Claude Code handles its own API connectivity. If it reports errors:
1. Output parser detects error patterns
2. Show in chat as system message (red background)
3. User decides whether to retry or end session

### File System Errors

If watched files become inaccessible:
1. Log warning
2. Show degraded state in UI (e.g., "Unable to read PROGRESS.md")
3. Continue operating — file watching resumes when files are accessible again

### Graceful Shutdown

When Helm closes:
1. Save all active session transcripts
2. Send SIGTERM to all Claude Code processes
3. Wait up to 5s for graceful exit
4. SIGKILL any remaining processes
5. Save app state

---

## 16. Build & Distribution

### macOS (Phase 1)

```
Build: cargo tauri build --target universal-apple-darwin
Output: Helm.app (universal binary: arm64 + x86_64)
Package: DMG with drag-to-Applications installer
Signing: Apple Developer certificate (for notarization)
Min OS: macOS 13 (Ventura) — WebView2 requirement
```

### Windows (Future)

```
Build: cargo tauri build --target x86_64-pc-windows-msvc
Output: Helm.exe
Package: NSIS installer or MSI
Requirement: WebView2 (bundled or bootstrapped)
```

### Linux (Future)

```
Build: cargo tauri build --target x86_64-unknown-linux-gnu
Output: AppImage + .deb
Requirement: WebKitGTK
```

### Auto-Updates

Tauri's built-in updater plugin:
- Check for updates on launch
- Download in background
- Show "Update available" toast
- Apply on next restart

---

## 17. Development Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Scaffold Tauri 2 + Svelte 5 + TailwindCSS project
- [ ] Set up project structure (all directories)
- [ ] Design system: colors, typography, spacing as CSS custom properties
- [ ] Animation system: spring presets, transition helpers
- [ ] i18n setup: svelte-i18n, en.json with initial keys
- [ ] Component library: Button, IconButton, Badge, Tooltip, Modal, Toast, ProgressBar
- [ ] Layout components: Sidebar, TopBar, MainLayout, Panel
- [ ] Rust: basic Tauri commands, app state structure
- [ ] Basic view routing (sidebar nav → swap views)

### Phase 2: Process Bridge (Week 3-4)
- [ ] Rust: ProcessManager — spawn, pipe stdin/stdout, kill
- [ ] Rust: OutputParser — basic text parsing, question detection
- [ ] Rust: Tauri commands for session management
- [ ] Rust: Tauri events for streaming output
- [ ] Frontend: session store, message store
- [ ] Test: spawn claude, send message, receive output

### Phase 3: Splash + Dashboard (Week 4-5)
- [ ] Splash screen with animated logo and loading sequence
- [ ] Dashboard view with all components
- [ ] Pipeline stepper (reads filesystem for doc existence)
- [ ] Quick action cards with animations
- [ ] Asset counters (scans project directory)
- [ ] Recent sessions list
- [ ] Sidebar navigation with all view links

### Phase 4: Chat Interface (Week 5-7)
- [ ] Chat view with message list
- [ ] ChatMessage component (user + agent variants, markdown rendering)
- [ ] ChatInput with multi-line, Cmd+Enter, disabled-while-typing
- [ ] AgentHeader with phase name
- [ ] ContextMeter (estimated)
- [ ] New Session / End Session flow with confirmations
- [ ] Session transcript persistence
- [ ] Integration: send /game-idea, conversation flows end-to-end

### Phase 5: Documents + Code (Week 7-8)
- [ ] Rust: FileWatcher for docs/ directory
- [ ] Documents view: tabs, markdown rendering, section nav, search
- [ ] Code browser: file tree, syntax-highlighted viewer
- [ ] Diff view component

### Phase 6: Orchestration Monitor (Week 8-10)
- [ ] Rust: PROGRESS.md parser
- [ ] Orchestration view: phase progress bar
- [ ] Agent grid with live status tiles
- [ ] Task kanban board
- [ ] Live log stream with filtering
- [ ] Hook monitor
- [ ] Pause/Stop/Resume controls
- [ ] Integration: run /orchestrate, monitor full execution

### Phase 7: Polish & Extras (Week 10-12)
- [ ] Git timeline view
- [ ] Session history view with search
- [ ] Settings view (models, parallelism, hooks, appearance)
- [ ] Keyboard shortcuts (Cmd+N, Cmd+W, Cmd+,)
- [ ] Error handling polish (all edge cases)
- [ ] Performance optimization (virtual scrolling for long logs)
- [ ] macOS DMG packaging
- [ ] App icon design

---

## 18. Open Questions

1. **Claude Code CLI flags**: Need to verify exact flags for non-interactive piped mode. Test `--print` vs `--verbose` vs `--output-format` to find the best combination for parsing.

2. **Context usage estimation**: Claude Code doesn't expose token counts in stdout. Options:
   - Count characters as rough estimate (1 token ≈ 4 chars)
   - Parse any token usage info Claude Code may print in verbose mode
   - Track message count as proxy

3. **Suggested actions in chat**: How smart should these be? Options:
   - Hardcoded per phase (e.g., genre options during game-idea)
   - Parsed from agent output (detect numbered lists, yes/no questions)
   - None in v1 (just free-text input)

4. **Splash artwork**: Need to design/commission the Helm logo. Should be:
   - Simple, geometric mark (helm wheel or compass rose)
   - Works at 16px (app icon) and 256px (splash)
   - SVG for crisp rendering and animation

5. **Offline capability**: Helm itself works offline, but Claude Code needs internet. Should we detect connectivity and show a clear "offline" state?

6. **Multiple projects**: v1 is single-project (one working directory). Future: project switcher with recent projects list.

---

## Summary

Helm is a Tauri 2.0 + Svelte 5 desktop app that wraps Claude Code CLI in a polished GUI. It manages sessions (one per pipeline phase), streams agent output into a chat interface, monitors multi-agent orchestration in real-time, and provides document/code browsing. Every string is localization-ready, every interaction is animated, and the design system enforces a consistent terminal-inspired aesthetic.

The user never touches a terminal. They click buttons, read chat bubbles, and watch progress bars. Claude Code does all the work underneath.
