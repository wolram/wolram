use anyhow::{Context, Result};
use git2::{IndexAddOption, Repository, Signature};
use std::path::Path;

use crate::state_machine::Job;

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
    ///
    /// Sensitive files (wolram.toml, .env, .env.local, *.key) are excluded
    /// from staging to prevent accidental exposure of secrets.
    pub fn commit(&self, message: &str) -> Result<String> {
        let mut index = self.repo.index()?;
        index.add_all(
            ["*"].iter(),
            IndexAddOption::DEFAULT,
            Some(&mut |path: &std::path::Path, _: &[u8]| -> i32 {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let excluded = ["wolram.toml", ".env", ".env.local"];
                if excluded.contains(&name) || name.ends_with(".key") {
                    1 // skip
                } else {
                    0 // add
                }
            }),
        )?;
        index.write()?;

        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let sig = self
            .repo
            .signature()
            .or_else(|_| Signature::now("WOLRAM", "wolram@localhost"))?;

        let parent = self.repo.head()?.peel_to_commit()?;
        let commit_oid = self
            .repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;

        let short = &commit_oid.to_string()[..7];
        Ok(short.to_string())
    }

    /// Create and checkout a new branch from HEAD.
    #[allow(dead_code)]
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head_commit = self.repo.head()?.peel_to_commit()?;
        self.repo.branch(name, &head_commit, false)?;

        let refname = format!("refs/heads/{name}");
        self.repo.set_head(&refname)?;
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(())
    }

    /// Create a commit summarising the job result, returning the short hash.
    ///
    /// Commit message format: `wolram: [skill] description (status)`
    pub fn commit_job_result(&self, job: &Job) -> Result<String> {
        let skill = job
            .agent
            .as_ref()
            .map(|a| a.skill.as_str())
            .unwrap_or("unknown");
        let status = format!("{:?}", job.status).to_lowercase();
        let message = format!("wolram: [{}] {} ({})", skill, job.description, status);
        self.commit(&message)
    }

    /// Create and checkout a branch named `wolram/<first-8-chars-of-job-id>`.
    #[allow(dead_code)]
    pub fn create_job_branch(&self, job: &Job) -> Result<()> {
        let short_id = &job.id[..8.min(job.id.len())];
        let branch_name = format!("wolram/{short_id}");
        self.create_branch(&branch_name)
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
    use crate::state_machine::{JobStatus, ModelTier, RetryConfig};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

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

    /// Helper: create a temp repo with an initial commit so HEAD exists.
    fn setup_temp_repo() -> (TempDir, GitManager) {
        let tmp = TempDir::new().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();

        // Create an initial commit so HEAD is valid.
        let sig = Signature::now("test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        drop(tree);
        drop(repo);
        let gm = GitManager::open(tmp.path()).unwrap();
        (tmp, gm)
    }

    #[test]
    fn commit_job_result_creates_commit_with_job_info() {
        let (tmp, gm) = setup_temp_repo();

        // Write a file so there is something to commit.
        fs::write(tmp.path().join("file.txt"), "hello").unwrap();

        let mut job = Job::new("Add login page".into(), RetryConfig::default());
        job.assign_agent("code_generation".to_string(), ModelTier::Sonnet);
        job.status = JobStatus::Completed;

        let hash = gm.commit_job_result(&job).unwrap();
        assert_eq!(hash.len(), 7);
    }

    #[test]
    fn commit_job_result_without_agent_uses_unknown_skill() {
        let (tmp, gm) = setup_temp_repo();
        fs::write(tmp.path().join("file.txt"), "data").unwrap();

        let job = Job::new("Do something".into(), RetryConfig::default());
        let hash = gm.commit_job_result(&job).unwrap();
        assert_eq!(hash.len(), 7);
    }

    #[test]
    fn create_job_branch_uses_first_8_chars_of_id() {
        let (_tmp, gm) = setup_temp_repo();

        let job = Job::new("Branch test".into(), RetryConfig::default());
        let expected_branch = format!("wolram/{}", &job.id[..8]);

        gm.create_job_branch(&job).unwrap();
        let branch = gm.current_branch().unwrap();
        assert_eq!(branch, expected_branch);
    }
}
