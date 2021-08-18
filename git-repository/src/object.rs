use std::{borrow::Borrow, cell::Ref, ops::DerefMut};

pub use git_object::Kind;

use crate::{
    hash::{oid, ObjectId},
    object,
    objs::immutable,
    odb,
    odb::{Find, FindExt},
    Access, Object, ObjectRef, Oid,
};

mod oid_impls {
    use super::Oid;
    use crate::{
        hash::{oid, ObjectId},
        Object, ObjectRef,
    };

    impl<'repo, A, B> PartialEq<Oid<'repo, A>> for Oid<'repo, B> {
        fn eq(&self, other: &Oid<'repo, A>) -> bool {
            self.id == other.id
        }
    }

    impl<'repo, A> PartialEq<ObjectId> for Oid<'repo, A> {
        fn eq(&self, other: &ObjectId) -> bool {
            &self.id == other
        }
    }

    impl<'repo, A> PartialEq<oid> for Oid<'repo, A> {
        fn eq(&self, other: &oid) -> bool {
            self.id == other
        }
    }

    impl<'repo, A, B> PartialEq<ObjectRef<'repo, A>> for Oid<'repo, B> {
        fn eq(&self, other: &ObjectRef<'repo, A>) -> bool {
            self.id == other.id
        }
    }

    impl<'repo, A> PartialEq<Object> for Oid<'repo, A> {
        fn eq(&self, other: &Object) -> bool {
            self.id == other.id
        }
    }

    impl<'repo, A> std::fmt::Debug for Oid<'repo, A> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.id.fmt(f)
        }
    }

    impl<'repo, A> AsRef<oid> for Oid<'repo, A> {
        fn as_ref(&self) -> &oid {
            &self.id
        }
    }

    impl<'repo, A> From<Oid<'repo, A>> for ObjectId {
        fn from(v: Oid<'repo, A>) -> Self {
            v.id
        }
    }
}

impl Object {
    pub fn attach<A>(self, access: &A) -> ObjectRef<'_, A>
    where
        A: Access + Sized,
    {
        *access.cache().buf.borrow_mut() = self.data;
        ObjectRef {
            id: self.id,
            kind: self.kind,
            data: Ref::map(access.cache().buf.borrow(), |v| v.as_slice()),
            access,
        }
    }
}

impl<'repo, A> ObjectRef<'repo, A>
where
    A: Access + Sized,
{
    pub(crate) fn from_current_buf(id: impl Into<ObjectId>, kind: Kind, access: &'repo A) -> Self {
        ObjectRef {
            id: id.into(),
            kind,
            data: Ref::map(access.cache().buf.borrow(), |v| v.as_slice()),
            access,
        }
    }
}

pub mod find {
    use crate::odb;

    pub type Error = odb::compound::find::Error;
    pub mod existing {
        use crate::odb;

        pub type Error = odb::pack::find::existing::Error<odb::compound::find::Error>;
    }
}

mod object_ref_impls {
    use crate::{Object, ObjectRef};

    impl<'repo, A> From<ObjectRef<'repo, A>> for Object {
        fn from(r: ObjectRef<'repo, A>) -> Self {
            r.into_owned()
        }
    }

    impl<'repo, A> AsRef<[u8]> for ObjectRef<'repo, A> {
        fn as_ref(&self) -> &[u8] {
            &self.data
        }
    }

    impl AsRef<[u8]> for Object {
        fn as_ref(&self) -> &[u8] {
            &self.data
        }
    }
}

impl<'repo, A> ObjectRef<'repo, A> {
    pub fn to_owned(&self) -> Object {
        Object {
            id: self.id,
            kind: self.kind,
            data: self.data.to_owned(),
        }
    }

    pub fn into_owned(self) -> Object {
        Object {
            id: self.id,
            kind: self.kind,
            data: self.data.to_owned(),
        }
    }

    pub fn detach(self) -> Object {
        self.into()
    }
}

impl<'repo, A> ObjectRef<'repo, A>
where
    A: Access + Sized,
{
    pub fn to_commit_iter(&self) -> Option<immutable::CommitIter<'_>> {
        odb::data::Object::new(self.kind, &self.data).into_commit_iter()
    }

    pub fn to_tag_iter(&self) -> Option<immutable::TagIter<'_>> {
        odb::data::Object::new(self.kind, &self.data).into_tag_iter()
    }
}

pub mod peel_to_kind {
    use crate::{
        object::{peel_to_kind, Kind},
        objs::immutable,
        odb, Access, ObjectRef,
    };

    impl<'repo, A> ObjectRef<'repo, A>
    where
        A: Access + Sized,
    {
        // TODO: tests
        pub fn peel_to_kind(mut self, kind: Kind) -> Result<Self, peel_to_kind::Error> {
            loop {
                match self.kind {
                    any_kind if kind == any_kind => {
                        return Ok(self);
                    }
                    Kind::Commit => {
                        let tree_id = self.to_commit_iter().expect("commit").tree_id().expect("valid commit");
                        let access = self.access;
                        drop(self);
                        self = crate::ext::access::object::find_existing_object(access, tree_id)?;
                    }
                    Kind::Tag => {
                        let target_id = self.to_tag_iter().expect("tag").target_id().expect("valid tag");
                        let access = self.access;
                        drop(self);
                        self = crate::ext::access::object::find_existing_object(access, target_id)?;
                    }
                    Kind::Tree | Kind::Blob => {
                        return Err(peel_to_kind::Error::NotFound {
                            actual: self.kind,
                            expected: kind,
                        })
                    }
                }
            }
        }
    }

    mod error {
        use quick_error::quick_error;

        use crate::{hash::ObjectId, object, object::find, odb};

        quick_error! {
            #[derive(Debug)]
            pub enum Error {
                FindExisting(err: find::existing::Error) {
                    display("A non existing object was encountered during object peeling")
                    from()
                    source(err)
                }
                NotFound{actual: object::Kind, expected: object::Kind} {
                    display("Last encountered object kind was {} while trying to peel to {}", actual, expected)
                }
            }
        }
    }
    pub use error::Error;
}

impl<'repo, A> Oid<'repo, A>
where
    A: Access + Sized,
{
    // NOTE: Can't access other object data that is attached to the same cache.
    pub fn existing_object(&self) -> Result<ObjectRef<'repo, A>, find::existing::Error> {
        crate::ext::access::object::find_existing_object(self.access, self.id)
    }

    // NOTE: Can't access other object data that is attached to the same cache.
    pub fn object(&self) -> Result<Option<ObjectRef<'repo, A>>, find::Error> {
        crate::ext::access::object::find_object(self.access, self.id)
    }
}

impl<'repo, A> Oid<'repo, A>
where
    A: Access + Sized,
{
    pub(crate) fn from_id(id: impl Into<ObjectId>, access: &'repo A) -> Self {
        Oid { id: id.into(), access }
    }

    pub fn detach(self) -> ObjectId {
        self.id
    }
}
