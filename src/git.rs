use anyhow::{Context, Result};
use git2::{IndexAddOption, Repository, Signature};
use std::path::Path;

pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    /// Open an existing git repository at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path).context("failed to open git repository")?;
        Ok(Self { repo })
    }

    /// Stage all changes and create a commit, returning the short hash.
    pub fn commit(&self, message: &str) -> Result<String> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let sig = self
            .repo
            .signature()
            .or_else(|_| Signature::now("WOLRAM", "wolram@localhost"))?;

        let parent = self.repo.head()?.peel_to_commit()?;
        let commit_oid =
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;

        let short = &commit_oid.to_string()[..7];
        Ok(short.to_string())
    }

    /// Create and checkout a new branch from HEAD.
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head_commit = self.repo.head()?.peel_to_commit()?;
        self.repo.branch(name, &head_commit, false)?;

        let refname = format!("refs/heads/{name}");
        self.repo.set_head(&refname)?;
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(())
    }

    /// Get the current branch name.
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let name = head
            .shorthand()
            .context("branch name is not valid UTF-8")?
            .to_string();
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn open_fails_on_non_repo_path() {
        let result = GitManager::open(&PathBuf::from("/tmp/definitely_not_a_repo_xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn open_succeeds_on_current_repo() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let gm = GitManager::open(&manifest_dir).expect("should open repo");
        let branch = gm.current_branch().expect("should get branch");
        assert!(!branch.is_empty());
    }
}
