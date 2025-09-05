# GEMINI Project Analysis: Git Statistics

## Project Overview

This project is a desktop application designed for developers to analyze and visualize their Git commit history. It's built using the Tauri framework, which allows for a web-based frontend with a Rust backend.

The application enables users to add local Git repositories, and it then processes the commit history to provide insights into coding habits and project activity. Key features include statistical charts (like 24-hour activity distribution and author contributions), a detailed commit timeline, and data caching in an SQLite database to ensure fast performance after the initial analysis.

**Core Technologies:**

*   **Backend:** Rust with Tauri, `git2-rs` for Git repository analysis, and `sqlx` for SQLite database operations.
*   **Frontend:** React with TypeScript, using Vite for the build tooling and ECharts for data visualization.
*   **Framework:** Tauri, for creating a cross-platform desktop application.

## Building and Running the Project

The project uses `npm` for managing frontend dependencies and running development scripts.

*   **Install Dependencies:**
    ```bash
    npm install
    ```

*   **Run in Development Mode:**
    This command starts the Vite dev server for the frontend and the Tauri application in development mode. It supports hot-reloading for the frontend.
    ```bash
    npm run tauri dev
    ```

*   **Build for Production:**
    This command builds and bundles the application for production.
    ```bash
    npm run tauri build
    ```

## Development Conventions

*   **Backend Commands:** The Rust backend exposes several commands to the frontend via the Tauri IPC bridge. These commands handle all the core logic, such as adding/removing repositories, scanning for commits, and querying the database for statistics. Key commands include `add_repository`, `get_statistics`, and `get_commit_timeline`.
*   **Database:** The application uses an SQLite database (`identifier.sqlite` in the root) to cache commit data, preventing the need to re-analyze entire repositories on each startup. The backend is responsible for all database interactions.
*   **Frontend Structure:** The React frontend is located in the `src` directory and is structured with components for different parts of the UI, such as `RepositoryManager`, `StatisticsCharts`, and `CommitDetailPage`.
*   **Styling:** The project uses standard CSS (`App.css`) for styling.
*   **State Management:** Frontend state, including repository lists and statistical data, is fetched from the backend and managed within the React components.
