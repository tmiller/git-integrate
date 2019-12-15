use git2::Remote;

#[derive(Debug)]
pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl Repo {
    pub fn new(remote: &Remote) -> Option<Repo> {
        let url = match remote.url() {
            Some(url) => url,
            None => return None,
        };
        let mut parts = url.split('/').rev();

        let name = match parts.next() {
            Some(name) => name.trim_end_matches(".git").to_string(),
            None => return None,
        };

        let owner = match parts.next().and_then(|s| s.split(':').rev().next()) {
            Some(owner) => owner.to_string(),
            None => return None,
        };
        Some(Repo { owner, name })
    }
}
