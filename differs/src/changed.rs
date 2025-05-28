
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Changed<'a, T: 'a> {
    Added(&'a T),
    Removed(&'a T),
    AddedAt(usize, &'a T, usize),
    RemovedAt(usize, &'a T, usize),
    Moved(&'a T, usize, usize),
    /// Element remained at its index but mutated in-place.
    ModifiedAt(usize, &'a T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapChanged<'a, K: 'a, V: 'a> {
    AddedEntry(&'a K, &'a V),
    RemovedEntry(&'a K, &'a V),
    ChangedEntry(&'a K),
}

/// Implemented automatically by **`#[derive(Diff)]`**.
pub trait HasChanges {
    type Change<'a>
    where
        Self: 'a;

    fn collect_changes<'a>(old: &'a Self, new: &'a Self, out: &mut Vec<Self::Change<'a>>)
    where
        Self: 'a;
}

/// Convenience helper.
#[inline]
pub fn diff_changes<'a, T: HasChanges>(old: &'a T, new: &'a T) -> Vec<T::Change<'a>> {
    let mut v = Vec::new();
    T::collect_changes(old, new, &mut v);
    v
}

/// `changed!` – flexible, typed diff‑matching macro with zero runtime cost.
///
/// * `@` immediately after the path targets the `self_` variant of the nested change enum.
///
#[macro_export]
macro_rules! changed {
    // Entry with at least one arm (consumes first, recurses on rest)
    (
        $change:expr;
        //  variant WITH `@` shorthand
        $ty:ident $( . $path:ident )* @ ( $pat:pat ) => $body:block
        $( ; $($rest:tt)* )?
    ) => {{
        $crate::__changed_arm_at! { $change, $ty $( . $path )*, $pat, $body }
        $( $crate::changed!($change; $($rest)* ); )?
    }};

    (
        $change:expr;
        $ty:ident $( . $path:ident )* ( $pat:pat ) => $body:block
        $( ; $($rest:tt)* )?
    ) => {{
        $crate::__changed_arm! { $change, $ty $( . $path )*, $pat, $body }
        $( $crate::changed!($change; $($rest)* ); )?
    }};

    // Empty remainder – terminate recursion
    ( $change:expr; ) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __changed_arm {
    // `.self(` alias
    ( $change:expr, $ty:ident . self, $pat:pat, $body:block ) => {
        if let $ty Change::self_($pat) = $change { $body }
    };

    // simple leaf
    ( $change:expr, $ty:ident . $field:ident, $pat:pat, $body:block ) => {
        paste::paste! {
            if let [<$ty Change>]::$field($pat) = $change { $body }
        }
    };

    // nested chain
    ( $change:expr, $ty:ident . $first:ident . $($tail:ident).+, $pat:pat, $body:block ) => {
        paste::paste! {
            if let [<$ty Change>]::[<$first>](inner) = $change {
                $crate::changed!(inner; [<$first:camel>] . $($tail).+($pat) => $body );
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __changed_arm_at {
    // direct field snapshot
    ( $change:expr, $ty:ident . $field:ident, $pat:pat, $body:block ) => {
        paste::paste! {
            if let [<$ty Change>]::$field(inner) = $change {
                if let [<$field:camel Change>]::self_($pat) = inner { $body }
            }
        }
    };

    // nested chain with snapshot at tail
    ( $change:expr, $ty:ident . $first:ident . $($tail:ident).+, $pat:pat, $body:block ) => {
        paste::paste! {
            if let [<$ty Change>]::[<$first>](inner) = $change {
                $crate::__changed_arm_at!( inner, [<$first:camel>] . $($tail).+, $pat, $body );
            }
        }
    };
}
