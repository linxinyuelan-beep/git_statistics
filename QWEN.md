# Git 工作量统计桌面应用 - 项目上下文 (QWEN.md)

## 项目概览

这是一个名为 "Git 工作量统计" 的桌面应用程序。其主要目的是分析本地 Git 仓库的提交历史，统计代码的增删行数，并以图表和时间线的形式可视化这些数据。该应用旨在帮助开发者和团队了解代码贡献情况。

**核心技术栈:**
*   **前端框架:** React (使用 TypeScript)
*   **后端框架:** Tauri (使用 Rust)
*   **数据库:** SQLite
*   **Git 分析库:** git2-rs (Rust)
*   **图表库:** ECharts (通过 echarts-for-react 在前端使用)
*   **构建工具:** Vite (前端), Cargo (Rust)

## 项目结构

```
git_statistics/
├── src/                    # React 前端代码
│   ├── components/         # React 组件 (如 RepositoryManager, StatisticsCharts, Timeline)
│   ├── types/              # TypeScript 类型定义
│   └── App.tsx            # 主应用组件
├── src-tauri/             # Rust 后端代码 (Tauri 应用逻辑)
│   ├── src/
│   │   ├── commands.rs    # Tauri 命令处理函数 (供前端调用的后端 API)
│   │   ├── database.rs    # SQLite 数据库操作 (使用 sqlx)
│   │   ├── git_analyzer.rs # Git 仓库分析逻辑 (使用 git2-rs)
│   │   ├── models.rs      # 数据模型定义
│   │   └── main.rs        # Tauri 应用主入口
│   └── Cargo.toml         # Rust 依赖和配置
├── package.json           # Node.js 前端依赖和脚本配置
└── README.md             # 项目详细说明文档
```

## 构建、运行与开发

### 环境要求
*   Node.js >= 16
*   Rust >= 1.60
*   Git

### 关键命令

*   **安装依赖:**
    ```bash
    npm install
    ```
*   **开发模式运行:**
    ```bash
    npm run tauri dev
    ```
    *   此命令会启动 Vite 开发服务器 (默认端口 1420) 并运行 Tauri 应用。
*   **构建生产版本:**
    ```bash
    npm run tauri build
    ```
    *   此命令会构建前端资源并使用 Tauri 打包成桌面应用安装包。

### 开发流程
1.  前端 (React/TypeScript) 通过 `@tauri-apps/api` 提供的 `invoke` 函数调用后端 Rust 定义的命令。
2.  后端 (Rust/Tauri) 处理这些命令，执行 Git 分析、数据库操作等任务，并将结果返回给前端。
3.  前端接收到数据后，使用 ECharts 进行可视化展示。

## 主要功能

### 仓库管理
*   用户可以通过文件选择对话框添加本地 Git 仓库路径。
*   系统会验证路径是否为有效的 Git 仓库。
*   添加的仓库信息存储在 SQLite 数据库中。
*   用户可以删除已添加的仓库。

### Git 数据分析
*   后端使用 `git2-rs` 库分析指定仓库的 Git 提交历史。
*   提取每次提交的作者、时间、新增行数、删除行数等信息。
*   为避免重复分析，系统会记录每个仓库的上次扫描时间，并只分析新增的提交。

### 数据存储与缓存
*   所有分析得到的提交数据存储在本地 SQLite 数据库中。
*   数据库位置：
    *   macOS: `~/Library/Application Support/com.gitstatistics.desktop/git_stats.db`
*   这种缓存机制大大加快了后续的数据加载和分析速度。

### 数据可视化
*   **统计图表 (StatisticsCharts 组件):**
    *   24小时分布图：展示一天内每小时的代码变更量。
    *   7天趋势图：显示最近一周的代码增删趋势。
    *   作者贡献图：TOP 10 贡献者的代码占比。
    *   仓库分布图：不同仓库的代码贡献对比。
*   **提交时间线 (Timeline 组件):**
    *   按时间顺序展示所有加载的提交记录。
*   所有图表均使用 ECharts 渲染。

### 数据筛选
*   用户可以在前端界面通过以下条件筛选展示的数据：
    *   时间范围 (开始日期, 结束日期)
    *   作者 (按提交者姓名筛选)
    *   仓库 (只分析特定仓库)
    *   提供了快速筛选按钮 (如过去7天、过去30天)。

## 关键 API 接口 (Tauri Commands)

前端通过 `invoke` 调用以下后端命令：

*   `add_repository(path: string)`: 添加一个新的 Git 仓库。
*   `remove_repository(id: number)`: 根据 ID 删除一个仓库。
*   `get_repositories()`: 获取所有已添加仓库的列表。
*   `scan_repository(repository_id: number)`: 分析指定仓库的 Git 历史并更新数据库。
*   `get_statistics(filters)`: 根据筛选条件获取聚合统计数据。
*   `get_commit_timeline(filters)`: 根据筛选条件获取详细的提交时间线数据。

## 开发规范与约定

*   **前端:** 使用 React Hooks 和 TypeScript。组件化开发。
*   **后端:** 使用 Rust 和 Tauri。通过 `commands.rs` 暴露 API 给前端。数据库操作使用 `sqlx`，Git 操作使用 `git2-rs`。
*   **数据流:** 前端发起请求 -> Tauri 命令处理 -> Rust 后端执行逻辑 (Git 分析/数据库查询) -> 返回数据 -> 前端更新状态并渲染 UI。
*   **数据库:** 使用 SQLite 本地存储，通过 `sqlx` 进行异步操作。数据模型在 `models.rs` 中定义。