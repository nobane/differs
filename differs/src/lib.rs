extern crate self as differs;

mod field_paths;
pub use field_paths::*;

pub use differs_derive::Diff;
pub use differs_derive::Fields;

mod changed;
pub use changed::*;
