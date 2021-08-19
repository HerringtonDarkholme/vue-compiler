use crate::core::TemplateCompiler;
use std::path::PathBuf;
use std::borrow::Cow;

pub enum PadOption {
    Line,
    Space,
    NoPad,
}

pub struct SFCParseOptions {
    pub filename: String,
    pub source_map: bool,
    pub source_root: PathBuf,
    pub pad: PadOption,
    pub ignore_empty: bool,
}

impl Default for SFCParseOptions {
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
