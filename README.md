# Git 工作量统计桌面应用

基于 Tauri + React + Rust 开发的 Git 工作量统计工具，帮助开发者分析和可视化代码提交数据。

## 功能特性

### 核心功能
- ✅ 仓库管理：添加、删除多个本地 Git 仓库
- ✅ 数据采集：分析 Git 提交历史，统计代码增删行数
- ✅ 智能缓存：使用 SQLite 本地缓存，避免重复分析
- ✅ 统计图表：24小时柱状图、7天趋势图、作者贡献饼图等
- ✅ 时间线视图：展示所有仓库的提交时间线
- ✅ 筛选功能：按作者、时间段、仓库筛选数据

### 可视化图表
- **24小时分布图**：展示一天内每小时的代码变更量
- **7天趋势图**：显示最近一周的代码增删趋势
- **作者贡献图**：TOP 10 贡献者的代码占比
- **仓库分布图**：不同仓库的代码贡献对比
- **提交时间线**：按时间顺序展示所有提交记录

## 技术栈

- **前端**：React + TypeScript + ECharts
- **后端**：Rust + Tauri
- **数据库**：SQLite
- **Git 分析**：git2-rs
- **样式**：原生 CSS

## 开发环境要求

- Node.js >= 16
- Rust >= 1.60
- Git

## 安装和运行

### 1. 克隆项目
```bash
git clone <repository-url>
cd git_statistics
```

### 2. 安装依赖
```bash
npm install
```

### 3. 开发模式运行
```bash
npm run tauri dev
```

### 4. 构建生产版本
```bash
npm run tauri build
```

## 使用指南

### 1. 添加仓库
- 点击左侧边栏的"添加仓库"按钮
- 选择包含 .git 文件夹的项目目录
- 系统会自动验证是否为有效的 Git 仓库

### 2. 分析数据
- 点击顶部的"刷新数据"按钮
- 系统会分析所有添加的仓库
- 首次分析可能较慢，后续会使用缓存加速

### 3. 查看统计
- **统计图表**：查看各种可视化图表
- **提交时间线**：浏览详细的提交历史
- 支持搜索和筛选功能

### 4. 数据筛选
- 按作者筛选：只显示特定作者的提交
- 按时间筛选：设置开始和结束日期
- 按仓库筛选：只分析特定仓库

## 项目结构

```
git_statistics/
├── src/                    # React 前端代码
│   ├── components/         # React 组件
│   ├── types/             # TypeScript 类型定义
│   └── App.tsx            # 主应用组件
├── src-tauri/             # Rust 后端代码
│   ├── src/
│   │   ├── commands.rs    # Tauri 命令
│   │   ├── database.rs    # 数据库操作
│   │   ├── git_analyzer.rs # Git 分析逻辑
│   │   ├── models.rs      # 数据模型
│   │   └── main.rs        # 主程序入口
│   └── Cargo.toml         # Rust 依赖配置
├── package.json           # Node.js 依赖配置
└── README.md             # 项目说明
```

## API 接口

### Tauri 命令
- `add_repository(path: string)` - 添加仓库
- `remove_repository(id: number)` - 删除仓库
- `get_repositories()` - 获取仓库列表
- `scan_repository(repository_id: number)` - 扫描仓库
- `get_statistics(filters)` - 获取统计数据
- `get_commit_timeline(filters)` - 获取提交时间线

## 数据存储

应用使用 SQLite 数据库存储分析结果，位置：
- macOS: `~/Library/Application Support/com.gitstatistics.desktop/git_stats.db`
- Windows: `%APPDATA%\\com.gitstatistics.desktop\\git_stats.db`
- Linux: `~/.local/share/com.gitstatistics.desktop/git_stats.db`

## 常见问题

### Q: 添加仓库时提示"不是有效的 Git 仓库"
A: 请确保选择的目录包含 .git 文件夹，或者是 Git 仓库的根目录。

### Q: 扫描大仓库很慢怎么办？
A: 首次扫描会分析所有历史提交，比较耗时。后续扫描只会处理新提交，速度会快很多。

### Q: 如何清理缓存数据？
A: 可以直接删除 SQLite 数据库文件，下次启动时会重新创建。

### Q: 支持远程仓库吗？
A: 当前版本只支持本地仓库，远程仓库支持将在未来版本中添加。

## 开发计划

### 已完成功能
- [x] 基础仓库管理
- [x] Git 数据采集和分析
- [x] SQLite 数据缓存
- [x] 基础统计图表
- [x] 提交时间线
- [x] 数据筛选功能

### 计划中功能
- [ ] 热力图日历视图
- [ ] 更多图表类型（雷达图、堆叠图等）
- [ ] 代码差异查看器
- [ ] 文件类型分析
- [ ] 导出功能（PDF、CSV、PNG）
- [ ] 主题切换
- [ ] 多语言支持
- [ ] 远程仓库支持

## 贡献指南

欢迎提交 Issue 和 Pull Request！

### 开发环境设置
1. Fork 本项目
2. 创建功能分支: `git checkout -b feature/新功能`
3. 提交更改: `git commit -am '添加新功能'`
4. 推送分支: `git push origin feature/新功能`
5. 创建 Pull Request

## 许可证

MIT License

## 作者

开发者

---

**注意**：本应用仅在本地运行，不会上传任何代码或数据到服务器，保护您的隐私安全。