use git_repository as git;

use crate::commit::Message;

/// A head reference will all commits that are 'governed' by it, that is are in its exclusive ancestry.
pub struct Segment<'a> {
    pub head: git::refs::Reference,
    /// only relevant history items, that is those that change code in the respective crate.
    pub history: Vec<&'a Item>,
}

pub struct Item {
    pub id: git::hash::ObjectId,
    pub message: Message,
    pub tree_data: Vec<u8>,
}
