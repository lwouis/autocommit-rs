#![feature(iterator_for_each)]

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
use fs_extra::dir::{copy, move_dir, CopyOptions};
use git2::{Repository, Direction};
use config::Config;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};

pub fn main() {
    env_logger::init().unwrap();
    if let Err(err) = autocommit() {
        error!("Autocommit failed: {}", err);
    }
}

fn autocommit() -> Result<(), Box<Error>> {
    let config: Config = serde_json::from_reader(File::open("resources/config.json")?)?;
    info!("{:#?}", config);
    let repo = Repository::open(&config.destination_repo)?;
    watch_and_commit(&repo, &config)?;
    Ok(())

    //    AutoCommit.REPO = await nodegit.Repository.open(AutoCommit.CONFIG.destinationRepo)
    //    chokidar.watch(AutoCommit.CONFIG.filesToWatch)
    //        .on('add', path => AutoCommit.add(path))
    //    .on('change', path => AutoCommit.update(path))
    //    .on('unlink', path => AutoCommit.remove(path))
    //    .on('unlinkDir', path => AutoCommit.remove(path))
    //    .on('error', error => { throw error })
}

fn watch_and_commit(repo: &Repository, config: &Config) -> Result<(), Box<Error>> {
    let (tx, rx) = channel();
    // delay helps handle composed events
    // see https://docs.rs/notify/4.0.1/notify/trait.Watcher.html#tymethod.new
    let delay = Duration::from_secs(2);
    let mut watcher: RecommendedWatcher = Watcher::new(tx, delay)?;
    &config.files_to_watch.iter().for_each(|file| {
        watcher.watch(file, RecursiveMode::Recursive).unwrap();
    });
    loop {
        match rx.recv()? {
            DebouncedEvent::Create(path) => {
                copy_file_or_dir_into_repo(&path, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Write(..) => {
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Remove(path) => {
                remove_file_or_dir_from_repo(&path, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            DebouncedEvent::Rename(path_from, path_to) => {
                rename_file_or_dir_in_repo(&path_from, &path_to, &repo.path())?;
                add_all_and_commit_and_push(&repo, &config)?;
            }
            // TODO handle DebouncedEvent::Chmod(path)
            _ => {}
        }
    }
}

fn copy_file_or_dir_into_repo(path: &PathBuf, repo: &Path) -> Result<(), Box<Error>> {
    let path_in_repo_buf = repo.join(path);
    let path_in_repo = path_in_repo_buf.as_path();
    match path_in_repo.is_dir() {
        true => {
            let options = CopyOptions {
                overwrite: true,
                skip_exist: false,
                buffer_size: 64000,
            };
            copy(path, path_in_repo, &options)?;
        }
        false => {
            fs::copy(path, path_in_repo)?;
        }
    }
    Ok(())
}

fn remove_file_or_dir_from_repo(path: &PathBuf, repo: &Path) -> Result<(), Box<Error>> {
    let path_in_repo_buf = repo.join(path);
    let path_in_repo = path_in_repo_buf.as_path();
    match path_in_repo.is_dir() {
        true => {
            fs::remove_dir_all(path_in_repo)?;
        }
        false => {
            fs::remove_file(path_in_repo)?;
        }
    }
    Ok(())
}

fn rename_file_or_dir_in_repo(from: &PathBuf, to: &PathBuf, repo: &Path) -> Result<(), Box<Error>> {
    let path_from_in_repo_buf = repo.join(from);
    let path_from_in_repo = path_from_in_repo_buf.as_path();
    match path_from_in_repo.is_dir() {
        true => {
            let options = CopyOptions {
                overwrite: true,
                skip_exist: false,
                buffer_size: 64000,
            };
            let path_to_in_repo_buf = repo.join(to);
            let path_to_in_repo = path_to_in_repo_buf.as_path();
            move_dir(path_from_in_repo, path_to_in_repo, &options)?;
        }
        false => {
            fs::remove_file(path_from_in_repo)?;
        }
    }

    remove_file_or_dir_from_repo(from, repo)?;
    copy_file_or_dir_into_repo(to, repo)?;
    Ok(())
}

fn add_all_and_commit_and_push(repo: &Repository, config: &Config) -> Result<(), Box<Error>> {
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
