type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

mod file;
mod edit {
    mod refeditext {
        use git_ref::edit::{Change, RefEdit, RefEditsExt};
        use std::convert::TryInto;

        fn named_edit(name: &str) -> RefEdit {
            RefEdit {
                edit: Change::Delete { previous: None },
                name: name.try_into().expect("valid name"),
            }
        }

        #[test]
        fn reject_duplicates() {
            assert!(
                (&[named_edit("HEAD")][..]).assure_one_name_has_one_edit().is_ok(),
                "there are no duplicates"
            );
            assert!(
                (&[named_edit("refs/foo"), named_edit("HEAD")][..])
                    .assure_one_name_has_one_edit()
                    .is_ok(),
                "there are no duplicates"
            );
            assert_eq!(
                (&[named_edit("HEAD"), named_edit("refs/heads/main"), named_edit("HEAD")][..])
                    .assure_one_name_has_one_edit()
                    .expect_err("duplicate"),
                "HEAD",
                "a correctly named duplicate"
            );
        }
    }
}
