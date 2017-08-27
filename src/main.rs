#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate env_logger;
extern crate serde;
extern crate serde_json;
extern crate git2;
extern crate time;
extern crate notify;

mod config;

use std::fs::File;
use std::path::Path;
use std::error::Error;
use git2::{Repository, Direction};
use config::Config;

fn config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

fn do_stuff() -> Result<(), Box<Error>> {
    let config = config_from_file("resources/config.json")?;
    info!("{:#?}", config);
    let repo = Repository::open(&config.destination_repo)?;
    add_all_and_commit_and_push(repo, config)?;
    Ok(())
    //    AutoCommit.REPO = await nodegit.Repository.open(AutoCommit.CONFIG.destinationRepo)
    //    chokidar.watch(AutoCommit.CONFIG.filesToWatch)
    //        .on('add', path => AutoCommit.add(path))
    //    .on('change', path => AutoCommit.update(path))
    //    .on('unlink', path => AutoCommit.remove(path))
    //    .on('unlinkDir', path => AutoCommit.remove(path))
    //    .on('error', error => { throw error })
}

fn add_all_and_commit_and_push(repo: Repository, config: Config) -> Result<(), Box<Error>> {
    // commit
    let mut index = repo.index()?;
    index.add_path(Path::new("."))?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let message = format!("Autocommit {}", time::now().rfc822());
    let head = repo.find_commit(repo.head().unwrap().target().unwrap())
        .unwrap();
    let signature = repo.signature()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&head],
    )?;
    // push
    let mut remote = repo.find_remote(&config.remote)?;
    remote.connect(Direction::Push)?;
    remote.push(&[&config.refs], None)?;
    Ok(())
}

fn main() {
    env_logger::init().unwrap();
    if let Err(err) = do_stuff() {
        error!("Autocommit failed: {}", err);
    }
}
