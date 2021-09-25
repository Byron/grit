//!
use crate::{
    easy,
    easy::{
        ext::ObjectAccessExt,
        object,
        object::{peel, Kind},
        ObjectRef,
    },
};

///
pub mod to_kind {
    mod error {

        use crate::easy::object;

        /// The error returned by [`Object::peel_to_kind()`][crate::easy::ObjectRef::peel_to_kind()].
        #[derive(Debug, thiserror::Error)]
        #[allow(missing_docs)]
        pub enum Error {
            #[error(transparent)]
            FindExistingObject(#[from] object::find::existing::Error),
            #[error("Last encountered object kind was {} while trying to peel to {}", .actual, .expected)]
            NotFound {
                actual: object::Kind,
                expected: object::Kind,
            },
        }
    }
    pub use error::Error;
}

impl<'repo, A> ObjectRef<'repo, A>
where
    A: easy::Access + Sized,
{
    // TODO: tests
    /// Follow tags to their target and commits to trees until the given `kind` of object is encountered.
    ///
    /// Note that this object doesn't necessarily have to be the end of the chain.
    /// Typical values are [`Kind::Commit`] or [`Kind::Tree`].
    pub fn peel_to_kind(mut self, kind: Kind) -> Result<Self, peel::to_kind::Error> {
        loop {
            match self.kind {
                any_kind if kind == any_kind => {
                    return Ok(self);
                }
                Kind::Commit => {
                    let tree_id = self
                        .try_to_commit_iter()
                        .expect("commit")
                        .tree_id()
                        .expect("valid commit");
                    let access = self.access;
                    drop(self);
                    self = access.find_object(tree_id)?;
                }
                Kind::Tag => {
                    let target_id = self.to_tag_iter().target_id().expect("valid tag");
                    let access = self.access;
                    drop(self);
                    self = access.find_object(target_id)?;
                }
                Kind::Tree | Kind::Blob => {
                    return Err(peel::to_kind::Error::NotFound {
                        actual: self.kind,
                        expected: kind,
                    })
                }
            }
        }
    }

    // TODO: tests
    /// Follow all tag object targets until a commit, tree or blob is reached.
    ///
    /// Note that this method is different from [`peel_to_kind(…)`][ObjectRef::peel_to_kind()] as it won't
    /// peel commits to their tree, but handles tags only.
    pub fn peel_tags_to_end(mut self) -> Result<Self, object::find::existing::Error> {
        loop {
            match self.kind {
                Kind::Commit | Kind::Tree | Kind::Blob => break Ok(self),
                Kind::Tag => {
                    let target_id = self.to_tag_iter().target_id().expect("valid tag");
                    let access = self.access;
                    drop(self);
                    self = access.find_object(target_id)?;
                }
            }
        }
    }
}
