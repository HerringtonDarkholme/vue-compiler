use smallvec::SmallVec;
use std::path::PathBuf;

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

pub struct SfcBlock {}

pub struct SfcTemplateBlock {}

pub struct SfcScriptBlock {}

pub struct SfcStyleBlock {}

pub struct SfcDescriptor<'a> {
    pub filename: String,
    pub source: &'a str,
    pub template: Option<SfcTemplateBlock>,
    pub scripts: SmallVec<[SfcScriptBlock; 1]>,
    pub styles: SmallVec<[SfcStyleBlock; 1]>,
    pub custom_blocks: Vec<SfcBlock>,
    pub css_vars: Vec<&'a str>,
    /// whether the SFC uses :slotted() modifier.
    /// this is used as a compiler optimization hint.
    pub slotted: bool,
}

pub enum SfcError {
    CompilerError,
    SyntaxError,
}

pub struct SfcParseResult<'a> {
    pub descriptor: SfcDescriptor<'a>,
    pub errors: Vec<SfcError>,
}

pub fn parse_sfc(_source: &str) -> SfcParseResult<'_> {
    todo!()
}
