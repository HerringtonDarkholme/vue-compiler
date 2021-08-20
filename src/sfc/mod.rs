use std::path::PathBuf;

mod parse_sfc;
mod descriptors;

pub use parse_sfc::{
    parse_sfc,
    SfcDescriptor,
};

pub enum PadOption {
    Line,
    Space,
    NoPad,
}

pub struct SfcParseOptions {
    pub filename: String,
    pub source_map: bool,
    pub source_root: PathBuf,
    pub pad: PadOption,
    pub ignore_empty: bool,
}

impl Default for SfcParseOptions {
    fn default() -> Self {
        Self {
            filename: "anonymous.vue".into(),
            source_map: true,
            source_root: "".into(),
            pad: PadOption::NoPad,
            ignore_empty: true,
        }
    }
}
