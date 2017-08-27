#[derive(Deserialize, Debug)]
pub struct Config {
    /// List of absolute paths of the files/folders to watch (recursively)
    /// e.g.  ["/path/config.conf", "/path/config-directory/"]
    pub files_to_watch: Vec<String>,
    /// Absolute path to the Git repository where the files are copied and committed
    /// e.g. /path/git_clones/repo
    pub destination_repo: String,
    /// Name of the remote on the Git repository where the files are copied and committed
    /// e.g. "origin"
    pub remote: String,
    /// Git references
    /// e.g. "refs/heads/master:refs/heads/master"
    pub refs: String,
}