use std::path::{Path, PathBuf};
use std::process::Command;

use crate::domain::git::{GitAvailability, GitBranch, GitDiffStat, GitLogEntry, GitStatus};
use crate::error::{CoreError, CoreResult};

pub fn git_available() -> GitAvailability {
    match run_git(None, &["--version"]) {
        Ok(output) => GitAvailability {
            available: true,
            version: Some(output.trim().to_owned()),
            error: None,
        },
        Err(error) => GitAvailability {
            available: false,
            version: None,
            error: Some(error.to_string()),
        },
    }
}

pub fn canonical_repo_path(raw: &str) -> CoreResult<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::validation(
            "path",
            "Repository path is required.",
        ));
    }
    let path = Path::new(trimmed)
        .canonicalize()
        .map_err(|_| CoreError::validation("path", "Repository path does not exist."))?;
    if !path.is_dir() {
        return Err(CoreError::validation(
            "path",
            "Repository path must be a directory.",
        ));
    }
    Ok(path)
}

pub fn ensure_git_repository(path: &Path) -> CoreResult<()> {
    let output = run_git(Some(path), &["rev-parse", "--show-toplevel"])?;
    if output.trim().is_empty() {
        return Err(CoreError::validation(
            "path",
            "This directory is not a Git repository.",
        ));
    }
    Ok(())
}

pub fn current_branch(path: &Path) -> CoreResult<String> {
    let output = run_git(Some(path), &["branch", "--show-current"])?;
    let branch = output.trim();
    if branch.is_empty() {
        return Err(CoreError::validation(
            "branch",
            "The repository is in detached HEAD state.",
        ));
    }
    Ok(branch.to_owned())
}

pub fn status(path: &Path) -> CoreResult<GitStatus> {
    let output = run_git(Some(path), &["status", "--porcelain=v1", "-b"])?;
    let mut lines = output.lines();
    let header = lines.next().unwrap_or_default();
    let branch = header
        .strip_prefix("## ")
        .unwrap_or(header)
        .split("...")
        .next()
        .unwrap_or("")
        .trim()
        .to_owned();
    let upstream = header
        .split("...")
        .nth(1)
        .and_then(|value| value.split(' ').next())
        .map(str::to_owned);
    let ahead = header
        .split("ahead ")
        .nth(1)
        .and_then(|value| value.split([',', ']']).next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let behind = header
        .split("behind ")
        .nth(1)
        .and_then(|value| value.split([',', ']']).next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let changes = lines
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    Ok(GitStatus {
        branch,
        upstream,
        ahead,
        behind,
        dirty: !changes.is_empty(),
        changes,
    })
}

pub fn branches(path: &Path) -> CoreResult<Vec<GitBranch>> {
    let current = current_branch(path)?;
    let output = run_git(
        Some(path),
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    )?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|name| GitBranch {
            name: name.to_owned(),
            current: name == current,
        })
        .collect())
}

pub fn log(path: &Path, limit: usize) -> CoreResult<Vec<GitLogEntry>> {
    let count = limit.clamp(1, 50).to_string();
    let output = run_git(
        Some(path),
        &[
            "log",
            &format!("-{count}"),
            "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%aI",
        ],
    )?;
    if output.trim().is_empty() {
        return Ok(vec![]);
    }
    Ok(output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            Some(GitLogEntry {
                hash: parts.next()?.to_owned(),
                short_hash: parts.next()?.to_owned(),
                subject: parts.next()?.to_owned(),
                author: parts.next()?.to_owned(),
                committed_at: parts.next()?.to_owned(),
            })
        })
        .collect())
}

pub fn diff_stat(path: &Path) -> CoreResult<GitDiffStat> {
    let output = run_git(Some(path), &["diff", "--stat"])?;
    let files_changed = output
        .lines()
        .last()
        .and_then(|line| line.split(" file").next())
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    let insertions = output
        .split(" insertion")
        .next()
        .and_then(|value| value.rsplit(',').next())
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    let deletions = output
        .split(" deletion")
        .next()
        .and_then(|value| value.rsplit(',').next())
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    Ok(GitDiffStat {
        files_changed,
        insertions,
        deletions,
        summary: output.trim().to_owned(),
    })
}

pub fn checkout_branch(path: &Path, branch: &str, create: bool) -> CoreResult<()> {
    validate_branch_name(branch)?;
    let status = status(path)?;
    if status.dirty {
        return Err(CoreError::validation(
            "worktree",
            "The working tree has uncommitted changes. Commit or stash them before switching branches.",
        ));
    }
    if create {
        run_git(Some(path), &["switch", "-c", branch])?;
    } else {
        run_git(Some(path), &["switch", branch])?;
    }
    Ok(())
}

pub fn validate_branch_name(branch: &str) -> CoreResult<()> {
    let name = branch.trim();
    if name.is_empty() || name.len() > 200 {
        return Err(CoreError::validation(
            "branch",
            "Branch name must be between 1 and 200 characters.",
        ));
    }
    if name.starts_with('-') || name.contains('\0') || name.contains('\n') {
        return Err(CoreError::validation("branch", "Branch name is invalid."));
    }
    // Ask Git itself; the argument is passed directly, never through a shell.
    run_git(None, &["check-ref-format", "--branch", name])?;
    Ok(())
}

pub fn remote_url(path: &Path) -> Option<String> {
    run_git(Some(path), &["remote", "get-url", "origin"])
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn run_git(path: Option<&Path>, args: &[&str]) -> CoreResult<String> {
    let mut command = Command::new("git");
    command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0");
    if let Some(path) = path {
        command.current_dir(path);
    }
    let output = command
        .output()
        .map_err(|error| CoreError::startup(format!("Git is not available: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(CoreError::validation(
            "git",
            if stderr.is_empty() {
                "Git command failed.".to_owned()
            } else {
                stderr
            },
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .status()
            .expect("git");
        assert!(status.success());
    }

    fn repo(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(name);
        std::fs::create_dir_all(&path).expect("dir");
        git(&path, &["init", "-b", "main"]);
        git(&path, &["config", "user.email", "test@example.com"]);
        git(&path, &["config", "user.name", "Test"]);
        std::fs::write(path.join("README.md"), "hello\n").expect("write");
        git(&path, &["add", "."]);
        git(&path, &["commit", "-m", "initial"]);
        (dir, path)
    }

    #[test]
    fn status_branches_log_and_dirty_guard() {
        let (_dir, path) = repo("space dir ü");
        ensure_git_repository(&path).expect("repo");
        let status = status(&path).expect("status");
        assert_eq!(status.branch, "main");
        assert!(!status.dirty);
        assert!(branches(&path).expect("branches").iter().any(|b| b.current));
        assert_eq!(log(&path, 5).expect("log").len(), 1);
        assert_eq!(diff_stat(&path).expect("diff").files_changed, 0);

        checkout_branch(&path, "feature/one", true).expect("create branch");
        assert_eq!(current_branch(&path).expect("branch"), "feature/one");
        checkout_branch(&path, "main", false).expect("switch main");

        std::fs::write(path.join("README.md"), "dirty\n").expect("dirty");
        assert!(checkout_branch(&path, "feature/one", false).is_err());
    }

    #[test]
    fn rejects_shell_injection_branch_names() {
        assert!(validate_branch_name("main;rm -rf /").is_err());
        assert!(validate_branch_name("feature/ok").is_ok());
        assert!(validate_branch_name("-danger").is_err());
    }
}
