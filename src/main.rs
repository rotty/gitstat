use std::fmt;

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
            Some(name) => {
                GitInfo::Branch {
                    branch: BranchInfo {
                        name: name.into(),
                        remote: None, // FIXME
                    },
                    status: Status::from_repo(repo)?,
                    oid: commit.id(),
                }
            }
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
    ahead: u32,
    behind: u32,
}

impl Distance {
    fn as_pair(self) -> (u32, u32) {
        (self.ahead, self.behind)
    }
}

fn info() -> anyhow::Result<GitInfo> {
    let repo = git2::Repository::discover(".")?;
    GitInfo::from_repo(&repo)
}

fn main() {
    match info() {
        Ok(info) => {
            print!("{}", info.prompt());
        }
        Err(_) => {
            // intentionally swallow error; TODO: add way to print error for
            // debugging.
        }
    }
}
