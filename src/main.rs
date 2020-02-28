use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Status {
    staged: u32,
    conflicts: u32,
    changed: u32,
    untracked: u32,
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
                    status: Status {
                        // FIXME
                        staged: 0,
                        conflicts: 0,
                        changed: 0,
                        untracked: 0,
                    },
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
            Branch {
                branch,
                status,
                ..
            } => {
                let (ahead, behind) = if let Some(remote) = &branch.remote {
                    remote.distance().map_or((0, 0), |d| d.as_pair())
                } else {
                    (0, 0)
                };
                write!(f, "{} {} {} {} {} {}", branch.name, ahead, behind, status.staged, status.conflicts, status.untracked)?;
            }
            Detached { oid } => {
                write!(f, ":{} 0 0 0 0 0", oid)?;
            }
            Unborn => {
                write!(f, "? 0 0 0 0 0")?;
            },
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
