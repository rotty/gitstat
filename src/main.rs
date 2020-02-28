use std::fmt;

use anyhow::anyhow;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Status {
    staged: u32,
    conflicts: u32,
    changed: u32,
    untracked: u32,
}

impl Status {
    fn from_repo(repo: &git2::Repository) -> anyhow::Result<Self> {
        let wt_changed_status = {
            use git2::Status;
            Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_TYPECHANGE | Status::WT_RENAMED
        };
        let index_changed_status = {
            use git2::Status;
            Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_TYPECHANGE
                | Status::INDEX_RENAMED
        };

        let mut staged = 0;
        let mut conflicts = 0;
        let mut changed = 0;
        let mut untracked = 0;

        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        for entry in repo.statuses(Some(&mut options))?.iter() {
            let status = entry.status();
            if status.is_conflicted() {
                conflicts += 1;
            }
            if status.intersects(wt_changed_status) {
                changed += 1;
            }
            if status.is_wt_new() {
                untracked += 1;
            }
            if status.intersects(index_changed_status) {
                staged += 1;
            }
        }
        Ok(Status {
            staged,
            conflicts,
            changed,
            untracked,
        })
    }
}

#[derive(Debug, Clone)]
struct BranchInfo {
    name: String,
    remote: Option<Remote>,
}

#[derive(Debug, Clone)]
struct Remote {
    branch: String,
    distance: Option<Distance>,
}

impl Remote {
    fn from_repo(repo: &git2::Repository, local_branch: &str) -> anyhow::Result<Option<Self>> {
        let branch = repo.find_branch(local_branch, git2::BranchType::Local)?;
        let upstream = match branch.upstream() {
            Ok(upstream) => upstream,
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };
        let local_commit = branch.get().peel_to_commit()?;
        let upstream_commit = upstream.get().peel_to_commit()?;
        let (ahead, behind) = repo.graph_ahead_behind(local_commit.id(), upstream_commit.id())?;
        Ok(Some(Remote {
            branch: upstream
                .name()?
                .ok_or_else(|| {
                    anyhow!(
                        "non-UTF8 upstream branch name for local branch  {}",
                        local_branch
                    )
                })?
                .into(),
            distance: Some(Distance { ahead, behind }),
        }))
    }
}

#[derive(Debug)]
enum GitInfo {
    Branch {
        branch: BranchInfo,
        status: Status,
        oid: git2::Oid,
    },
    Detached {
        oid: git2::Oid,
    },
    Unborn,
}

impl GitInfo {
    fn from_repo(repo: &git2::Repository) -> anyhow::Result<Self> {
        let head = match repo.head() {
            Ok(head) => head,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => return Ok(GitInfo::Unborn),
            Err(e) => return Err(e.into()),
        };
        let commit = head.peel_to_commit()?;
        let info = match head.shorthand() {
            Some(name) => GitInfo::Branch {
                branch: BranchInfo {
                    name: name.into(),
                    remote: Remote::from_repo(repo, name)?,
                },
                status: Status::from_repo(repo)?,
                oid: commit.id(),
            },
            None => {
                unimplemented!();
            }
        };
        Ok(info)
    }
    fn prompt(&self) -> Prompt<'_> {
        Prompt { info: self }
    }
}

struct Prompt<'a> {
    info: &'a GitInfo,
}

impl<'a> fmt::Display for Prompt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GitInfo::*;
        match self.info {
            Branch { branch, status, .. } => {
                let (ahead, behind) = if let Some(remote) = &branch.remote {
                    remote.distance().map_or((0, 0), |d| d.as_pair())
                } else {
                    (0, 0)
                };
                write!(
                    f,
                    "{} {} {} {} {} {} {}",
                    branch.name,
                    ahead,
                    behind,
                    status.staged,
                    status.conflicts,
                    status.changed,
                    status.untracked
                )?;
            }
            Detached { oid } => {
                write!(f, ":{} 0 0 0 0 0 0", oid)?;
            }
            Unborn => {
                write!(f, "? 0 0 0 0 0 0")?;
            }
        }
        Ok(())
    }
}

impl Remote {
    fn distance(&self) -> Option<Distance> {
        self.distance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Distance {
    ahead: usize,
    behind: usize,
}

impl Distance {
    fn as_pair(self) -> (usize, usize) {
        (self.ahead, self.behind)
    }
}

fn info() -> anyhow::Result<GitInfo> {
    let repo = git2::Repository::discover(".")?;
    GitInfo::from_repo(&repo)
}

// TODO: use environment variable or command-line option here
const DEBUG: bool = true;

fn main() {
    let rc = match info() {
        Ok(info) => {
            print!("{}", info.prompt());
            0
        }
        Err(e) => {
            if DEBUG {
                eprintln!("error: {}", e);
            }
            1
        }
    };
    std::process::exit(rc);
}
