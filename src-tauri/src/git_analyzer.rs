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
        
        let mut analyzer = GitAnalyzer {
            repo,
            repository_info,
            commit_to_branches: HashMap::new(),
        };
        
        // Build commit to branches mapping
        analyzer.build_commit_branch_mapping()?;
        
        Ok(analyzer)
    }

    fn build_commit_branch_mapping(&mut self) -> Result<()> {
        // Get current branch name
        let current_branch = self.get_current_branch_name();
        
        // Process local branches
        if let Ok(branches) = self.repo.branches(Some(git2::BranchType::Local)) {
            for branch_result in branches {
                if let Ok((branch, _)) = branch_result {
                    if let Ok(Some(branch_name)) = branch.name() {
                        // Walk the branch history
                        let branch_ref = branch.get();
                        if let Some(oid) = branch_ref.target() {
                            let mut revwalk = self.repo.revwalk()?;
                            revwalk.push(oid)?;
                            revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;
                            
                            for commit_oid in revwalk {
                                if let Ok(commit_oid) = commit_oid {
                                    self.commit_to_branches
                                        .entry(commit_oid)
                                        .or_insert_with(Vec::new)
                                        .push(branch_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Process remote branches
        if let Ok(branches) = self.repo.branches(Some(git2::BranchType::Remote)) {
            for branch_result in branches {
                if let Ok((branch, _)) = branch_result {
                    if let Ok(Some(branch_name)) = branch.name() {
                        // Clean up remote branch name (remove origin/ prefix)
                        let clean_name = branch_name.strip_prefix("origin/").unwrap_or(branch_name);
                        
                        // Walk the branch history
                        let branch_ref = branch.get();
                        if let Some(oid) = branch_ref.target() {
                            let mut revwalk = self.repo.revwalk()?;
                            revwalk.push(oid)?;
                            revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;
                            
                            for commit_oid in revwalk {
                                if let Ok(commit_oid) = commit_oid {
                                    let branches = self.commit_to_branches
                                        .entry(commit_oid)
                                        .or_insert_with(Vec::new);
                                    
                                    // Only add remote branch if no local branch exists
                                    if !branches.iter().any(|b| b == clean_name) {
                                        branches.push(format!("origin/{}", clean_name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
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
        let oid = git2::Oid::from_str(commit_id)?;
        let commit = self.repo.find_commit(oid)?;
        
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let message = commit.message().unwrap_or("").to_string();
        
        let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .unwrap_or_default();

        // Get current branch name if possible
        let branch = self.get_commit_branch(&commit)?;

        // Get remote URL
        let remote_url = self.get_remote_url();

        // Calculate diff stats and get file changes
        let (additions, deletions, files_changed, file_changes) = self.get_detailed_commit_stats(&commit)?;

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
        
        // Collect file changes with diffs
        let mut file_changes: Vec<FileChange> = Vec::new();
        
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
        
        if let Some(branches) = self.commit_to_branches.get(&commit_id) {
            if branches.is_empty() {
                return Ok("".to_string());
            }
            
            // Priority selection logic:
            // 1. Current branch (if commit belongs to it)
            if let Some(ref current) = current_branch {
                if branches.contains(current) {
                    return Ok(current.clone());
                }
            }
            
            // 2. Local non-main branches (prefer feature branches)
            for branch in branches {
                if !branch.starts_with("origin/") && 
                   !matches!(branch.as_str(), "main" | "master" | "develop" | "dev") {
                    return Ok(branch.clone());
                }
            }
            
            // 3. Remote non-main branches
            for branch in branches {
                if branch.starts_with("origin/") {
                    let clean_name = branch.strip_prefix("origin/").unwrap_or(branch);
                    if !matches!(clean_name, "main" | "master" | "develop" | "dev") {
                        return Ok(clean_name.to_string());
                    }
                }
            }
            
            // 4. Local main branches
            for branch in branches {
                if !branch.starts_with("origin/") && 
                   matches!(branch.as_str(), "main" | "master" | "develop" | "dev") {
                    return Ok(branch.clone());
                }
            }
            
            // 5. Remote main branches
            for branch in branches {
                if branch.starts_with("origin/") {
                    let clean_name = branch.strip_prefix("origin/").unwrap_or(branch);
                    if matches!(clean_name, "main" | "master" | "develop" | "dev") {
                        return Ok(clean_name.to_string());
                    }
                }
            }
            
            // 6. Fallback to first branch
            if let Some(first_branch) = branches.first() {
                let clean_name = first_branch.strip_prefix("origin/").unwrap_or(first_branch);
                return Ok(clean_name.to_string());
            }
        }
        
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