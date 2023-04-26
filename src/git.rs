use git2::{Diff, DiffOptions, Repository};

pub fn get_diff_after_add(repo: &Repository) -> Result<Diff, git2::Error> {
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;
    let head_tree = head_commit.tree()?;

    let index = repo.index()?;
    let index_tree = index.into_tree()?;

    let mut diff_options = DiffOptions::new();
    repo.diff_tree_to_index(Some(&head_tree), Some(&index_tree), Some(&mut diff_options))
}
