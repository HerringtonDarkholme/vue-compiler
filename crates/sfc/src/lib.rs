pub mod descriptors;
pub mod parse_sfc;
mod rewrite_default;
mod script;
mod style;
mod template;

// API
pub use parse_sfc::parse_sfc;
pub use script::compile_script;
pub use template::compile_template;
pub use style::compile_style;
pub use rewrite_default::rewrite_default;

// Structs
pub use parse_sfc::{
    SfcParseOptions, SfcDescriptor, SfcBlock, SfcScriptBlock, SfcTemplateBlock, SfcStyleBlock,
};
