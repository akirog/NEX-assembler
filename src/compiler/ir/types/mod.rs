pub const NONE_TYPE: &'static str = "void";

pub mod written_type;
pub mod resolved_type;
pub mod type_table;
pub mod field_types;

pub use resolved_type::*;
pub use written_type::*;