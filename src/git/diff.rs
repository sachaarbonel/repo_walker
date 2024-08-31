use gix::diff::tree::{Changes, Recorder, State};
use gix::objs::TreeRefIter;
use gix::Repository;

use gix::diff::tree::recorder::Change;

pub fn diff_trees<'a>(
    repo: &'a Repository,
    previous_tree: TreeRefIter,
    current_tree: TreeRefIter,
) -> Result<Vec<Change>, Box<dyn std::error::Error>> {
    let db = &repo.objects;

    let mut recorder = Recorder::default();
    Changes::from(previous_tree).needed_to_obtain(
        current_tree,
        &mut State::default(),
        db,
        &mut recorder,
    )?;
    Ok(recorder.records)
}
