//!
use std::{cell::Ref, convert::TryInto};

use git_hash::ObjectId;
pub use git_object::Kind;

use crate::{
    easy,
    easy::{Object, ObjectRef, TreeRef},
};

mod errors;
pub(crate) mod cache {
    pub use git_pack::cache::object::MemoryCappedHashmap;
}
pub use errors::{find, write};
mod impls;
pub mod peel;
mod tree;

impl Object {
    /// Infuse this owned object with an [`easy::Access`].
    pub fn attach<A>(self, access: &A) -> easy::borrow::state::Result<ObjectRef<'_, A>>
    where
        A: easy::Access + Sized,
    {
        *access.state().try_borrow_mut_buf()? = self.data;
        Ok(ObjectRef {
            id: self.id,
            kind: self.kind,
            data: Ref::map(access.state().try_borrow_buf()?, |v| v.as_slice()),
            access,
        })
    }
}

impl<'repo, A> ObjectRef<'repo, A>
where
    A: easy::Access + Sized,
{
    pub(crate) fn from_current_buf(
        id: impl Into<ObjectId>,
        kind: Kind,
        access: &'repo A,
    ) -> easy::borrow::state::Result<Self> {
        Ok(ObjectRef {
            id: id.into(),
            kind,
            data: Ref::map(access.state().try_borrow_buf()?, |v| v.as_slice()),
            access,
        })
    }

    /// Transform this object into a tree, or panic if it is none.
    pub fn into_tree(self) -> TreeRef<'repo, A> {
        match self.try_into() {
            Ok(tree) => tree,
            Err(this) => panic!("Tried to use {} as tree, but was {}", this.id, this.kind),
        }
    }

    /// Transform this object into a tree, or return it as part of the `Err` if it is no tree.
    pub fn try_into_tree(self) -> Result<TreeRef<'repo, A>, Self> {
        self.try_into()
    }
}

impl<'repo, A> ObjectRef<'repo, A> {
    /// Create an owned instance of this object, copying our data in the process.
    pub fn to_owned(&self) -> Object {
        Object {
            id: self.id,
            kind: self.kind,
            data: self.data.to_owned(),
        }
    }

    /// Turn this instance into an owned one, copying our data in the process.
    pub fn into_owned(self) -> Object {
        Object {
            id: self.id,
            kind: self.kind,
            data: self.data.to_owned(),
        }
    }

    /// Sever the connection to `Easy` and turn this instance into a standalone object.
    ///
    /// Note that the data buffer will be copied in the process.
    pub fn detach(self) -> Object {
        self.into()
    }
}

impl<'repo, A> ObjectRef<'repo, A>
where
    A: easy::Access + Sized,
{
    /// Obtain a fully parsed commit whose fields reference our data buffer, or panic if this is not a commit.
    pub fn to_commit(&self) -> git_object::CommitRef<'_> {
        self.try_to_commit().expect("can be decoded").expect("is a commit")
    }

    /// Obtain a fully parsed commit whose fields reference our data buffer.
    pub fn try_to_commit(&self) -> Result<Option<git_object::CommitRef<'_>>, git_object::decode::Error> {
        Ok(git_odb::data::Object::new(self.kind, &self.data)
            .decode()?
            .into_commit())
    }

    /// Obtain a an iterator over commit tokens like in [`to_commit_iter()`][ObjectRef::try_to_commit_iter()], but panic if this is not a commit.
    pub fn to_commit_iter(&self) -> git_object::CommitRefIter<'_> {
        git_odb::data::Object::new(self.kind, &self.data)
            .try_into_commit_iter()
            .expect("BUG: This object must be a commit")
    }

    /// Obtain a commit token iterator from the data in this instance, if it is a commit.
    pub fn try_to_commit_iter(&self) -> Option<git_object::CommitRefIter<'_>> {
        git_odb::data::Object::new(self.kind, &self.data).try_into_commit_iter()
    }

    /// Obtain a tag token iterator from the data in this instance, or panic if it is not a tag
    pub fn to_tag_iter(&self) -> git_object::TagRefIter<'_> {
        git_odb::data::Object::new(self.kind, &self.data)
            .try_into_tag_iter()
            .expect("BUG: this object must be a tag")
    }

    /// Obtain a tag token iterator from the data in this instance, if it is a tag.
    pub fn try_to_tag_iter(&self) -> Option<git_object::TagRefIter<'_>> {
        git_odb::data::Object::new(self.kind, &self.data).try_into_tag_iter()
    }
}
