use octocrab::Octocrab;

pub async fn repository_exists(
    octocrab: &Octocrab,
    repo_owner: &str,
    repo_name: &str,
) -> Result<bool, octocrab::Error> {
    match octocrab.repos(repo_owner, repo_name).get().await {
        Ok(_) => Ok(true),
        Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => Ok(false),
        Err(e) => Err(e),
    }
}
