use super::model::{CorpusFile, RepoEntry};

#[derive(Debug, Clone, Default)]
pub struct PruneStats {
    pub before: usize,
    pub after: usize,
    pub removed_errors: usize,
    pub removed_unresolved: usize,
}

pub fn prune_corpus(
    corpus: &mut CorpusFile,
    remove_errors: bool,
    remove_unresolved: bool,
) -> PruneStats {
    let before = corpus.repos.len();
    let mut removed_errors = 0usize;
    let mut removed_unresolved = 0usize;

    corpus.repos.retain(|repo| {
        if remove_errors && repo.resolve_error.is_some() {
            removed_errors += 1;
            return false;
        }
        if remove_unresolved && !repo.resolved && repo.resolve_error.is_none() {
            removed_unresolved += 1;
            return false;
        }
        true
    });

    PruneStats {
        before,
        after: corpus.repos.len(),
        removed_errors,
        removed_unresolved,
    }
}

pub fn unresolved_repos(corpus: &CorpusFile) -> Vec<&RepoEntry> {
    corpus
        .repos
        .iter()
        .filter(|r| !r.resolved && r.resolve_error.is_none())
        .collect()
}
