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
extern crate fs_extra;

mod config;

use std::path::{Path, PathBuf};
use std::error::Error;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::fs;
use std::fs::File;
use fs_extra::dir;
use fs_extra::dir::CopyOptions;
use git2::{Repository, Direction, Oid, Index};
use config::Config;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};

type Result<T> = std::result::Result<T, Box<Error>>;

pub fn main() {
    env_logger::init().expect("Log library failed to initialize");
    autocommit().unwrap_or_else(|err| {
        error!("Autocommit failed: {}", err);
    });
}

fn autocommit() -> Result<()> {
    let config: Config = serde_json::from_reader(File::open("resources/config.json")?)?;
    info!("{:#?}", config);
    let repo = Repository::open(&config.destination_repo)?;
    watch_and_commit(&repo, &config)
}

fn watch_and_commit(repo: &Repository, config: &Config) -> Result<()> {
    let (sender, receiver) = channel();
    // delay helps handle composed events
    // see https://docs.rs/notify/4.0.1/notify/trait.Watcher.html#tymethod.new
    let mut watcher: RecommendedWatcher = Watcher::new(sender, Duration::from_secs(2))?;
    for file in &config.files_to_watch {
        watcher.watch(file, RecursiveMode::Recursive)?;
    }
    loop {
        match receiver.recv()? {
            DebouncedEvent::Write(..) => {
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Create(path) => {
                copy_file_or_dir_into_repo(path, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Remove(path) => {
                remove_file_or_dir_from_repo(path, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Rename(path_from, path_to) => {
                rename_file_or_dir_in_repo(path_from, path_to, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            // not interested in other events
            _ => {}
            // TODO handle DebouncedEvent::Chmod(path)
        }
    }
}

fn copy_file_or_dir_into_repo(path: PathBuf, repo: &Path) -> Result<()> {
    let path_in_repo = repo.join(path.as_path());
    if path_in_repo.is_dir() {
        let options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 64000,
        };
        dir::copy(path, path_in_repo, &options)?;
    } else {
        fs::copy(path, path_in_repo)?;
    }
    Ok(())
}

fn remove_file_or_dir_from_repo(path: PathBuf, repo: &Path) -> Result<()> {
    let path_in_repo = repo.join(path.as_path());
    if path_in_repo.is_dir() {
        fs::remove_dir_all(path_in_repo)?;
    } else {
        fs::remove_file(path_in_repo)?;
    }
    Ok(())
}

fn rename_file_or_dir_in_repo(from: PathBuf, to: PathBuf, repo: &Path) -> Result<()> {
    let path_from_in_repo = repo.join(from.as_path());
    if path_from_in_repo.is_dir() {
        let options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 64000,
        };
        let path_to_in_repo = repo.join(to.as_path());
        dir::move_dir(path_from_in_repo, path_to_in_repo, &options)?;
    } else {
        fs::remove_file(path_from_in_repo)?;
    }
    remove_file_or_dir_from_repo(from, repo)?;
    copy_file_or_dir_into_repo(to, repo)
}

fn add_all_and_commit_and_push(repo: &Repository, config: &Config) -> Result<()> {
    let index = repo.index()?;
    let oid = add_all(index)?;
    commit(&repo, oid)?;
    push(&repo, &config)?;
    Ok(())
}

fn add_all(mut index: Index) -> Result<Oid> {
    index.add_path(Path::new("."))?;
    let oid = index.write_tree()?;
    Ok(oid)
}

fn commit(repo: &Repository, oid: Oid) -> Result<()> {
    let tree = repo.find_tree(oid)?;
    let message = format!("Autocommit {}", time::now().rfc822());
    let head_oid = repo.head()?.target().ok_or("Can't find head")?;
    let head = repo.find_commit(head_oid)?;
    let signature = repo.signature()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&head],
    )?;
    Ok(())
}

fn push(repo: &Repository, config: &Config) -> Result<()> {
    let mut remote = repo.find_remote(&config.remote)?;
    remote.connect(Direction::Push)?;
    remote.push(&[&config.refs], None)?;
    Ok(())
}
