# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-05

### Added

- Unity agent automation pipeline with rules, skills, and hooks
- Pipeline bar component in the dashboard layout
- `.claude` config injection into Unity project directories
- Orchestration timeline view
- `/catch-up` command for generating codebase comprehension guides
- `/init-project` command for game-specific CLAUDE.md generation

### Changed

- Enforce no XML docs or comments policy in agent templates
- Restructure dashboard layout to accommodate pipeline bar

## [0.1.0] - 2026-03-15

### Added

- Tauri desktop app with Rust backend and Svelte 5 frontend
- Chat session interface with per-message CLI execution and JSON stream parsing
- Orchestration dashboard with progress tracking
- Code browser, git timeline, and document viewer
- Settings persistence with frontend/backend serialization
- macOS build script and GitHub Actions CI pipeline
- Frontend test infrastructure with vitest
- Error recovery and offline detection
- Full permissions for spawned agents
- Project README with features, pipeline, and tech stack overview
- Build Game pipeline phase with embedded prompt content
- Command-line argument support in CLI path configuration

### Changed

- Enforce RectTransform and TextMeshPro rules across all agent prompts

### Fixed

- Orchestration tab not detecting PROGRESS.md outside docs/ directory
- Settings serialization mismatch between frontend and backend
- Splash screen display issues
