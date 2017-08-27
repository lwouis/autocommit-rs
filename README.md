# Purpose

Autocommit is a Rust app which watches a list of files/folder, and commits/pushes a copy to a Git repository when 
they are updated. Typical use-case would be to track configuration files (e.g. dotfiles) on Github automatically.

# How to install

* If you don't have it already, install [Rust](https://www.rust-lang.org)
* `git clone` this repo
* Run `cargo build --release`

# How to use

* Update the [config.json](resources/config.json) file
* Run the app in `target/release/`