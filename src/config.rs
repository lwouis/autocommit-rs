#[derive(Deserialize, Debug)]
pub struct Config {
    /// List of absolute paths of the files/folders to watch (recursively)
    /// e.g.  ["/path/config.conf", "/path/config-directory/"]
    files_to_watch: Vec<String>,
    /// Absolute path to the Git repository where the files are copied and committed
    /// e.g. /path/git_clones/repo
    destination_repo: String,
    /// Name of the remote on the Git repository where the files are copied and committed
    /// e.g. "origin"
    remote: String,
    /// Git references
    /// e.g. "refs/heads/master:refs/heads/master"
    refs: String,
}

impl Config {
    pub fn files_to_watch(&self) -> Vec<String> {
        let ref v = self.files_to_watch;
        v.clone()
    }
    pub fn destination_repo(&self) -> String {
        let ref s = self.destination_repo;
        s.clone()
    }
    pub fn remote(&self) -> String {
        let ref s = self.remote;
        s.clone()
    }
    pub fn refs(&self) -> String {
        let ref s = self.refs;
        s.clone()
    }
}
