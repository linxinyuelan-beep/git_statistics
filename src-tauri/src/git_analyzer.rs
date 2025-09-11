use git2::{Repository as GitRepository, DiffOptions, DiffFormat, DiffLineType, Oid};
use crate::models::{Commit, Repository};
use anyhow::{Result, Context};
use std::path::Path;
use std::collections::{HashSet, HashMap};

// New struct to hold file change information
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub additions: i32,
    pub deletions: i32,
    pub diff: String,
}

#[derive(Debug, Clone)]
pub struct AnalyzedCommit {
    pub commit: Commit,
    pub file_changes: Vec<FileChange>,
}

pub struct GitAnalyzer {
    repo: GitRepository,
    repository_info: Repository,
    commit_to_branches: HashMap<Oid, Vec<String>>,
}

impl GitAnalyzer {
    pub fn new(repository_info: Repository) -> Result<Self> {
        let repo = GitRepository::open(&repository_info.path)
            .context(format!("Failed to open git repository at {}", repository_info.path))?;
        
        let analyzer = GitAnalyzer {
            repo,
            repository_info,
            commit_to_branches: HashMap::new(), // 延迟初始化，只在需要时构建
        };
        
        Ok(analyzer)
    }

    fn get_current_branch_name(&self) -> Option<String> {
        if let Ok(head) = self.repo.head() {
            if let Some(name) = head.shorthand() {
                return Some(name.to_string());
            }
        }
        None
    }

    pub fn get_remote_url(&self) -> Option<String> {
        // Try to get the origin remote URL
        if let Ok(remote) = self.repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return Some(url.to_string());
            }
        }
        
        // If origin doesn't exist, try to get any remote URL
        if let Ok(remotes) = self.repo.remotes() {
            for remote_name in remotes.iter().flatten() {
                if let Ok(remote) = self.repo.find_remote(remote_name) {
                    if let Some(url) = remote.url() {
                        return Some(url.to_string());
                    }
                }
            }
        }
        
        None
    }

    pub fn analyze_commits(&self, since: Option<chrono::DateTime<chrono::Utc>>) -> Result<Vec<AnalyzedCommit>> {
        let mut revwalk = self.repo.revwalk()?;
        // Push all local branches instead of just HEAD
        revwalk.push_glob("refs/heads/*")?;
        // Also push all remote branches
        revwalk.push_glob("refs/remotes/*")?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        let mut processed_commits = HashSet::new();

        for oid_result in revwalk {
            let oid = oid_result?;
            
            if processed_commits.contains(&oid) {
                continue;
            }
            processed_commits.insert(oid);

            let commit = self.repo.find_commit(oid)?;
            
            // Skip if commit is before the 'since' time
            if let Some(since_time) = since {
                let commit_time = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_default();
                
                if commit_time < since_time {
                    break;
                }
            }

            // Skip merge commits to avoid double counting
            if commit.parent_count() > 1 {
                continue;
            }

            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            let author_email = author.email().unwrap_or("").to_string();
            let message = commit.message().unwrap_or("").to_string();
            
            let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .unwrap_or_default();

            // Get current branch name if possible
            let branch = self.get_commit_branch(&commit)?;

            // Calculate diff stats and get file changes
            let (additions, deletions, files_changed, file_changes) = self.get_detailed_commit_stats(&commit)?;

            let commit_data = Commit {
                id: oid.to_string(),
                repository_id: self.repository_info.id,
                repository_name: self.repository_info.name.clone(),
                author: author_name,
                email: author_email,
                message,
                timestamp,
                additions,
                deletions,
                files_changed,
                branch: Some(branch),
                remote_url: None, // This will be filled when retrieving from database
            };

            commits.push(AnalyzedCommit {
                commit: commit_data,
                file_changes,
            });
        }

        Ok(commits)
    }

    fn get_commit_stats(&self, commit: &git2::Commit) -> Result<(i32, i32, i32)> {
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.ignore_whitespace(true);
        diff_opts.ignore_blank_lines(true);

        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        let stats = diff.stats()?;
        
        Ok((
            stats.insertions() as i32,
            stats.deletions() as i32,
            stats.files_changed() as i32,
        ))
    }

    pub fn get_commit_detail(&self, commit_id: &str) -> Result<crate::models::CommitDetail> {
        let start_time = std::time::Instant::now();
        println!("🔧 开始获取commit详情: {}", &commit_id[..8]);
        
        let oid = git2::Oid::from_str(commit_id)?;
        let commit = self.repo.find_commit(oid)?;
        println!("📝 找到commit对象耗时: {:?}", start_time.elapsed());
        
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let message = commit.message().unwrap_or("").to_string();
        
        let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .unwrap_or_default();

        // Get current branch name if possible
        let branch_start = std::time::Instant::now();
        let branch = self.get_commit_branch(&commit)?;
        println!("🌿 获取分支信息耗时: {:?}", branch_start.elapsed());

        // Get remote URL
        let remote_start = std::time::Instant::now();
        let remote_url = self.get_remote_url();
        println!("🌍 获取远程URL耗时: {:?}", remote_start.elapsed());

        // Calculate diff stats and get file changes
        let diff_start = std::time::Instant::now();
        let (additions, deletions, files_changed, file_changes) = self.get_detailed_commit_stats(&commit)?;
        println!("📊 计算diff统计耗时: {:?}, 文件数: {}", diff_start.elapsed(), file_changes.len());

        // Convert FileChange to models::FileChange
        let model_file_changes = file_changes.into_iter().map(|fc| crate::models::FileChange {
            path: fc.path,
            additions: fc.additions,
            deletions: fc.deletions,
            diff: fc.diff,
        }).collect();

        Ok(crate::models::CommitDetail {
            id: commit_id.to_string(),
            repository_id: self.repository_info.id,
            repository_name: self.repository_info.name.clone(),
            author: author_name,
            email: author_email,
            message,
            timestamp,
            additions,
            deletions,
            files_changed,
            branch: Some(branch),
            remote_url,
            file_changes: model_file_changes,
        })
    }

    fn get_detailed_commit_stats(&self, commit: &git2::Commit) -> Result<(i32, i32, i32, Vec<FileChange>)> {
        let start_time = std::time::Instant::now();
        println!("📈 开始计算详细diff统计");
        
        let tree_start = std::time::Instant::now();
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };
        println!("🌳 获取tree对象耗时: {:?}", tree_start.elapsed());

        let diff_create_start = std::time::Instant::now();
        let mut diff_opts = DiffOptions::new();
        diff_opts.ignore_whitespace(true);
        diff_opts.ignore_blank_lines(true);

        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;
        println!("🔄 创建diff对象耗时: {:?}", diff_create_start.elapsed());

        let stats_start = std::time::Instant::now();
        let stats = diff.stats()?;
        println!("📊 获取基础统计耗时: {:?}", stats_start.elapsed());
        
        // Collect file changes with diffs
        let mut file_changes: Vec<FileChange> = Vec::new();
        
        let print_start = std::time::Instant::now();
        println!("🖨️  开始生成diff内容");
        
        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            let file_path = delta.new_file().path().or(delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            // Find or create file change entry
            let file_change = file_changes.iter_mut().find(|fc| fc.path == file_path);
            
            if let Some(file_change) = file_change {
                // Update existing file change
                match line.origin_value() {
                    DiffLineType::Addition => file_change.additions += 1,
                    DiffLineType::Deletion => file_change.deletions += 1,
                    _ => {}
                }
                
                // Append line to diff
                match line.origin_value() {
                    DiffLineType::Addition => {
                        file_change.diff.push_str(&format!("+{}", String::from_utf8_lossy(line.content())));
                    },
                    DiffLineType::Deletion => {
                        file_change.diff.push_str(&format!("-{}", String::from_utf8_lossy(line.content())));
                    },
                    DiffLineType::Context => {
                        file_change.diff.push_str(&format!(" {}", String::from_utf8_lossy(line.content())));
                    },
                    _ => {
                        file_change.diff.push_str(&format!(" {}", String::from_utf8_lossy(line.content())));
                    }
                }
            } else {
                // Create new file change entry
                let mut diff_content = String::new();
                match line.origin_value() {
                    DiffLineType::Addition => {
                        diff_content.push_str(&format!("+{}", String::from_utf8_lossy(line.content())));
                    },
                    DiffLineType::Deletion => {
                        diff_content.push_str(&format!("-{}", String::from_utf8_lossy(line.content())));
                    },
                    DiffLineType::Context => {
                        diff_content.push_str(&format!(" {}", String::from_utf8_lossy(line.content())));
                    },
                    _ => {
                        diff_content.push_str(&format!(" {}", String::from_utf8_lossy(line.content())));
                    }
                }
                
                file_changes.push(FileChange {
                    path: file_path,
                    additions: if line.origin_value() == DiffLineType::Addition { 1 } else { 0 },
                    deletions: if line.origin_value() == DiffLineType::Deletion { 1 } else { 0 },
                    diff: diff_content,
                });
            }
            
            true
        })?;
        
        println!("🖨️  生成diff内容耗时: {:?}", print_start.elapsed());
        println!("📈 总详细统计耗时: {:?}", start_time.elapsed());

        Ok((
            stats.insertions() as i32,
            stats.deletions() as i32,
            stats.files_changed() as i32,
            file_changes
        ))
    }

    fn get_commit_branch(&self, commit: &git2::Commit) -> Result<String> {
        let commit_id = commit.id();
        let current_branch = self.get_current_branch_name();
        
        // 优化：只检查HEAD分支是否指向该提交或其祖先
        // 这样避免了为每个分支执行完整的revwalk，大大提高性能
        
        // 检查当前HEAD分支
        if let Ok(head) = self.repo.head() {
            if let Ok(head_commit) = head.peel_to_commit() {
                if head_commit.id() == commit_id {
                    if let Some(head_name) = head.shorthand() {
                        return Ok(head_name.to_string());
                    }
                }
                
                // 检查是否在当前分支的历史中（只检查有限的提交）
                if let Some(oid) = head.target() {
                    let mut revwalk = self.repo.revwalk()?;
                    revwalk.push(oid)?;
                    revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;
                    
                    // 限制搜索深度，避免在大型仓库中耗时过长
                    let mut count = 0;
                    const MAX_DEPTH: usize = 1000;
                    
                    for commit_oid in revwalk {
                        if let Ok(commit_oid) = commit_oid {
                            if commit_oid == commit_id {
                                if let Some(head_name) = head.shorthand() {
                                    return Ok(head_name.to_string());
                                }
                            }
                        }
                        
                        count += 1;
                        if count >= MAX_DEPTH {
                            break;
                        }
                    }
                }
            }
        }
        
        // 如果HEAD不匹配，尝试其他本地分支（同样限制搜索深度）
        if let Ok(branches) = self.repo.branches(Some(git2::BranchType::Local)) {
            for branch_result in branches {
                if let Ok((branch, _)) = branch_result {
                    if let Ok(Some(branch_name)) = branch.name() {
                        // 跳过HEAD分支，因为我们已经检查过了
                        if let Ok(head) = self.repo.head() {
                            if let Some(head_name) = head.shorthand() {
                                if branch_name == head_name {
                                    continue;
                                }
                            }
                        }
                        
                        // 检查该分支是否直接指向此提交
                        if let Ok(branch_commit) = branch.get().peel_to_commit() {
                            if branch_commit.id() == commit_id {
                                return Ok(branch_name.to_string());
                            }
                            
                            // 限制搜索深度以提高性能
                            let branch_ref = branch.get();
                            if let Some(oid) = branch_ref.target() {
                                let mut revwalk = self.repo.revwalk()?;
                                revwalk.push(oid)?;
                                revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;
                                
                                let mut count = 0;
                                const MAX_DEPTH: usize = 100;
                                
                                for commit_oid in revwalk {
                                    if let Ok(commit_oid) = commit_oid {
                                        if commit_oid == commit_id {
                                            return Ok(branch_name.to_string());
                                        }
                                    }
                                    
                                    count += 1;
                                    if count >= MAX_DEPTH {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 简化处理：如果找不到精确匹配，返回空字符串
        // 在commit detail页面中，分支信息不是关键信息
        Ok("".to_string())
    }

    pub fn is_valid_git_repo(path: &str) -> bool {
        Path::new(path).join(".git").exists() || 
        GitRepository::open(path).is_ok()
    }
}

pub fn analyze_repository(repository: Repository, since: Option<chrono::DateTime<chrono::Utc>>) -> Result<Vec<AnalyzedCommit>> {
    if !GitAnalyzer::is_valid_git_repo(&repository.path) {
        return Err(anyhow::anyhow!("Path is not a valid git repository: {}", repository.path));
    }

    let analyzer = GitAnalyzer::new(repository)?;
    analyzer.analyze_commits(since)
}

// Static method to get remote URL for a repository path
pub fn get_remote_url_for_path(repo_path: &str) -> Option<String> {
    if let Ok(repo) = git2::Repository::open(repo_path) {
        // Try to get the origin remote URL
        if let Ok(remote) = repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return Some(url.to_string());
            }
        }
        
        // If origin doesn't exist, try to get any remote URL
        if let Ok(remotes) = repo.remotes() {
            for remote_name in remotes.iter().flatten() {
                if let Ok(remote) = repo.find_remote(remote_name) {
                    if let Some(url) = remote.url() {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }
    
    None
}