use smallvec::SmallVec;

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
