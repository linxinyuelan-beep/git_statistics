use git2::{Repository as GitRepository, DiffOptions, DiffFormat, DiffLineType};
use crate::models::{Commit, Repository};
use anyhow::{Result, Context};
use std::path::Path;
use std::collections::HashSet;

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
}

impl GitAnalyzer {
    pub fn new(repository_info: Repository) -> Result<Self> {
        let repo = GitRepository::open(&repository_info.path)
            .context(format!("Failed to open git repository at {}", repository_info.path))?;
        
        Ok(GitAnalyzer {
            repo,
            repository_info,
        })
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
        // For performance reasons, we only identify branches for head commits
        // This avoids expensive history walks for every commit
        
        // Check local branches first
        if let Ok(branches) = self.repo.branches(Some(git2::BranchType::Local)) {
            for branch_result in branches {
                if let Ok((branch, _)) = branch_result {
                    // Check if this is the head commit of the branch
                    if let Ok(branch_commit) = branch.get().peel_to_commit() {
                        if branch_commit.id() == commit.id() {
                            if let Ok(Some(name)) = branch.name() {
                                return Ok(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Check remote branches
        if let Ok(branches) = self.repo.branches(Some(git2::BranchType::Remote)) {
            for branch_result in branches {
                if let Ok((branch, _)) = branch_result {
                    // Check if this is the head commit of the branch
                    if let Ok(branch_commit) = branch.get().peel_to_commit() {
                        if branch_commit.id() == commit.id() {
                            if let Ok(Some(name)) = branch.name() {
                                // Clean up remote branch name (remove origin/ prefix)
                                let clean_name = name.strip_prefix("origin/").unwrap_or(name);
                                return Ok(clean_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // For non-head commits, we return None to indicate no specific branch
        // This will result in not showing branch badges for historical commits
        // which is better than showing "unknown" for everything
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