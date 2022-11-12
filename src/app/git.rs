use git2::Repository;
use std::fs::create_dir_all;
use std::path::Path;

pub fn clone_repository(repo: &str, path: &Path) -> Result<Repository, git2::Error> {
    create_dir_all(path).expect("Failed to create directory for post files");
    println!("Cloning {} to {}", repo, path.to_str().unwrap());
    Repository::clone(repo, path)
}

pub fn pull_repository(path: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    repo.find_remote("origin")?
        .fetch(&["main".to_string()], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_up_to_date() {
        Ok(())
    } else if analysis.0.is_fast_forward() {
        // TODO add functionality to pull from a different branch
        let refname = "refs/heads/main".to_string();
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
    } else {
        Err(git2::Error::from_str("Fast-forward only!"))
    }
}
