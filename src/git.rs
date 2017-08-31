extern crate time;

use git2::{Repository, Direction, Oid, Index};
use std::error::Error;
use std::result;
use std::path::Path;

use config::Config;

type Result<T> = result::Result<T, Box<Error>>;

pub struct Git {
    repo: Repository,
    config: Config,
    index: Index,
}

impl Git {
    pub fn new(config: Config) -> Result<Git> {
        let path = config.destination_repo();
        let repo = Repository::open(path)?;
        Ok(Git {
            config: config,
            index: repo.index()?,
            repo: repo,
        })
    }

    pub fn add_all_and_commit_and_push(&mut self) -> Result<()> {
        let oid = self.add_all()?;
        self.commit(oid)?;
        self.push()?;
        Ok(())
    }

    fn add_all(&mut self) -> Result<Oid> {
        self.index.add_path(Path::new("."))?;
        let oid = self.index.write_tree()?;
        Ok(oid)
    }

    fn commit(&self, oid: Oid) -> Result<()> {
        let tree = self.repo.find_tree(oid)?;
        let message = format!("Autocommit {}", time::now().rfc822());
        let head_oid = self.repo.head()?.target().ok_or("Can't find head")?;
        let head = self.repo.find_commit(head_oid)?;
        let signature = self.repo.signature()?;
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&head],
        )?;
        Ok(())
    }

    fn push(&self) -> Result<()> {
        let mut remote = self.repo.find_remote(&self.config.remote())?;
        remote.connect(Direction::Push)?;
        remote.push(&[&self.config.refs()], None)?;
        Ok(())
    }
}
