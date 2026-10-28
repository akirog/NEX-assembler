pub const NONE_TYPE: &'static str = "void";

pub mod unresolved_type;
pub mod resolved_type;

pub use resolved_type::*;
pub use unresolved_type::*;