# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Development
- `npm run dev` - Start Vite development server
- `npm run tauri dev` - Start Tauri development mode (recommended for desktop app development)

### Building
- `npm run build` - Build frontend (TypeScript compilation + Vite build)
- `npm run tauri build` - Build production desktop application

### Preview
- `npm run preview` - Preview built frontend

## Architecture Overview

This is a **Tauri desktop application** for Git repository statistics analysis with:

**Frontend (React + TypeScript)**:
- `src/` - React application with TypeScript
- `src/components/` - Main UI components (RepositoryManager, StatisticsCharts, Timeline)
- `src/types/index.ts` - TypeScript interfaces for data models
- `src/hooks/` and `src/utils/` - React hooks and utilities
- Visualization with ECharts via echarts-for-react

**Backend (Rust + Tauri)**:
- `src-tauri/src/` - Rust backend code
- `src-tauri/src/main.rs` - Application entry point with Tauri command handlers
- `src-tauri/src/commands.rs` - Tauri command implementations
- `src-tauri/src/git_analyzer.rs` - Git repository analysis logic using git2-rs
- `src-tauri/src/database.rs` - SQLite database operations using sqlx
- `src-tauri/src/models.rs` - Rust data structures

**Key Dependencies**:
- Frontend: React, TypeScript, ECharts, dayjs, react-router-dom, react-syntax-highlighter
- Backend: Tauri, git2, sqlx (SQLite), chrono, tokio, anyhow

## Tauri Commands (Frontend-Backend Interface)

The app exposes these Rust functions to the frontend via Tauri:
- `add_repository(path: string)` - Add Git repository
- `remove_repository(id: number)` - Remove repository
- `get_repositories()` - List all repositories
- `scan_repository(repository_id: number)` - Analyze repository commits
- `force_scan_repository(repository_id: number)` - Force re-scan repository (ignores cache)
- `get_statistics(filters)` - Get aggregated statistics
- `get_commit_timeline(filters)` - Get commit timeline data
- `get_commit_detail(commit_hash: string)` - Get detailed commit information

## Data Flow

1. **Repository Management**: Users add local Git repositories via directory picker
2. **Data Collection**: Git analysis scans commit history using git2-rs
3. **Caching**: Results stored in SQLite database to avoid re-analysis
4. **Visualization**: Statistics displayed via ECharts (hourly/daily trends, author contributions, etc.)
5. **Filtering**: Data can be filtered by author, date range, repository

## Database Schema

Uses SQLite with tables for:
- `repositories` - Repository metadata
- `commits` - Individual commit data with statistics
- Caching prevents re-scanning unchanged repositories

## Development Notes

- Tauri configuration in `src-tauri/tauri.conf.json`
- App runs on localhost:1420 in development (configured in vite.config.ts)
- Desktop app with window size 1200x800 (min 800x600)
- File system access enabled for repository selection (`dialog-open`, `fs-all` features)
- No remote repository support (local only)
- SQLite database stored as `identifier.sqlite` in project root
- Async state management with `AppState` struct to track scanning operations
- TypeScript strict mode enabled with comprehensive type checking