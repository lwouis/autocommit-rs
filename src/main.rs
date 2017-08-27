#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate env_logger;
extern crate serde;
extern crate serde_json;

mod config;

use std::fs::File;
use std::path::Path;
use std::error::Error;
use config::Config;

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
    let file = File::open(path)?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
}

fn do_stuff() -> Result<(), Box<Error>> {
    let config = read_config_from_file("resources/config.json")?;
    println!("{:#?}", config);
    Ok(())

    //    AutoCommit.REPO = await nodegit.Repository.open(AutoCommit.CONFIG.destinationRepo)
    //    chokidar.watch(AutoCommit.CONFIG.filesToWatch)
    //        .on('add', path => AutoCommit.add(path))
    //    .on('change', path => AutoCommit.update(path))
    //    .on('unlink', path => AutoCommit.remove(path))
    //    .on('unlinkDir', path => AutoCommit.remove(path))
    //    .on('error', error => { throw error })
}

fn main() {
    env_logger::init().unwrap();
    if let Err(err) = do_stuff() {
        error!("Autocommit failed: {}", err);
    }
}