
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

/// Users shouldn't need to touch this directly; use the `Fields` derive
/// and the `HasFields` trait generated for their own types.
///
/// Trait implemented for every type that derives `Fields`.
///
/// Obtain the *root* builder with `Foo::fields()` and then chain the
/// generated methods (`.bar().baz()…`) to build dotted paths.
///
pub trait HasFields {
    type Fields;

    /// Entry-point into the build-chain (`Foo::fields()`).
    fn fields() -> Self::Fields;
}

/// A single dotted path (e.g. `"database.url"` or `"a"`).
///
/// Created for you by `Foo::fields().some_field()`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FieldName(String);

impl FieldName {
    /// Get the dotted string path (borrowed).
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Build a `Field` from a `&'static str` literal with zero allocation.
    pub fn static_lit(lit: &'static str) -> Self {
        FieldName(lit.into())
    }

    /// Build a `Field` from an owned `String` (one allocation already paid).
    pub fn from_string(s: String) -> Self {
        FieldName(s)
    }

    /// Append `key` to the (possibly empty) `prefix`.
    pub fn join(prefix: &str, key: &'static str) -> Self {
        if prefix.is_empty() {
            FieldName::static_lit(key)
        } else if key.is_empty() {
            // used by `.self_()` on explicit proxies
            FieldName::from_string(prefix.to_owned())
        } else {
            FieldName::from_string(format!("{prefix}.{key}"))
        }
    }
}

/// Convert something that *represents* a path into a concrete [`FieldName`].
pub trait AsField {
    fn as_field(&self) -> FieldName;
}

impl AsField for FieldName {
    fn as_field(&self) -> FieldName {
        self.clone()
    }
}
impl AsField for &str {
    fn as_field(&self) -> FieldName {
        FieldName::from_string(self.to_string())
    }
}
impl AsField for String {
    fn as_field(&self) -> FieldName {
        FieldName::from_string(self.clone())
    }
}

#[derive(Default, Clone)]
pub struct ChangeEventBus {
    inner: Arc<Mutex<Registry>>,
}

#[derive(Default)]
struct Registry {
    subs: HashMap<String, Vec<Sender<String>>>,
}

impl ChangeEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to a particular field path.
    pub fn subscribe(&mut self, field: FieldName) -> Receiver<String> {
        let path = field.0.clone(); // keep the string alive inside the registry
        let (tx, rx) = mpsc::channel();
        self.inner
            .lock()
            .unwrap()
            .subs
            .entry(path)
            .or_default()
            .push(tx);
        rx
    }

    /// Push a change onto the bus (e.g. `"a"` or `"b.c"`).
    pub fn publish(&self, path: &str, new_value: String) {
        if let Some(list) = self.inner.lock().unwrap().subs.get_mut(path) {
            list.retain(|tx| tx.send(new_value.clone()).is_ok());
        }
    }
}
