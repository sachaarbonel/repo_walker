use gix::objs::Find;
use gix::{objs::TreeRefIter, Repository};
use std::path::Path;

pub fn open_repo(dir: impl AsRef<Path>) -> Result<Repository, Box<dyn std::error::Error>> {
    let git = gix::open::Options::isolated()
        .filter_config_section(|_| false)
        .open(dir.as_ref())?;

    Ok(git.to_thread_local())
}

pub fn find_revision<'a>(
    repo: &'a Repository,
    revision_name: &str,
) -> Result<gix::Object<'a>, Box<dyn std::error::Error>> {
    match repo.rev_parse_single(revision_name) {
        Ok(id) => repo.find_object(id).map_err(|e| {
            format!(
                "Failed to find object for revision '{}': {}",
                revision_name, e
            )
            .into()
        }),
        Err(e) => Err(format!("Failed to resolve revision '{}': {}", revision_name, e).into()),
    }
}

pub fn find_tree<'a>(
    repo: &'a Repository,
    obj: gix::Object<'a>,
    buf: &'a mut Vec<u8>,
) -> Result<TreeRefIter<'a>, Box<dyn std::error::Error>> {
    let db = &repo.objects;
    let tree = obj.peel_to_tree()?;
    let tree_id = tree.id();
    let data = db.try_find(&tree_id, buf).unwrap().unwrap();
    let tree = data.try_into_tree_iter().unwrap();
    Ok(tree)
}
