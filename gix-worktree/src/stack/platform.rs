use std::path::Path;

use bstr::ByteSlice;

use crate::stack::Platform;

/// Access
impl<'a> Platform<'a> {
    /// The full path to `relative` will be returned for use on the file system.
    pub fn path(&self) -> &'a Path {
        self.parent.stack.current()
    }

    /// See if the currently set entry is excluded as per exclude and git-ignore files.
    ///
    /// Note that this threats both classes, [*trashable*](gix_ignore::Kind::Expendable) and [*precious*](gix_ignore::Kind::Precious)
    /// as equal. If you need to differentiate, use [`matching_exclude_pattern()`](Self::matching_exclude_pattern)
    /// or [`excluded_kind()`](Self::excluded_kind).
    ///
    /// # Panics
    ///
    /// If the cache was configured without exclude patterns.
    #[doc(alias = "is_path_ignored", alias = "git2")]
    pub fn is_excluded(&self) -> bool {
        self.matching_exclude_pattern()
            .is_some_and(|m| !m.pattern.is_negative())
    }

    /// See if a non-negative ignore-pattern matches and obtain the kind of exclude, or return `None`
    /// if the path isn't excluded.
    ///
    /// This is similar to [`is_excluded()`](Self::is_excluded), but provides details that are useful to
    /// decide what to do with the excluded item.
    pub fn excluded_kind(&self) -> Option<gix_ignore::Kind> {
        self.matching_exclude_pattern()
            .and_then(|m| (!m.pattern.is_negative()).then_some(m.kind))
    }

    /// Check all exclude patterns to see if the currently set path matches any of them.
    ///
    /// Note that this pattern might be negated, and means this path in included.
    ///
    /// # Panics
    ///
    /// If the cache was configured without exclude patterns.
    pub fn matching_exclude_pattern(&self) -> Option<gix_ignore::search::Match<'_>> {
        let ignore = self.parent.state.ignore_or_panic();
        let relative_path =
            gix_path::to_unix_separators_on_windows(gix_path::into_bstr(self.parent.stack.current_relative()));
        ignore.matching_exclude_pattern(relative_path.as_bstr(), self.is_dir, self.parent.case)
    }

    /// Match all attributes at the current path and store the result in `out`, returning `true` if at least one attribute was found.
    ///
    /// # Panics
    ///
    /// If the cache was configured without attributes.
    #[cfg(feature = "attributes")]
    pub fn matching_attributes(&self, out: &mut gix_attributes::search::Outcome) -> bool {
        let attrs = self.parent.state.attributes_or_panic();
        let relative_path =
            gix_path::to_unix_separators_on_windows(gix_path::into_bstr(self.parent.stack.current_relative()));
        attrs.matching_attributes(relative_path.as_bstr(), self.parent.case, self.is_dir, out)
    }
}

impl std::fmt::Debug for Platform<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.path(), f)
    }
}
