use smallvec::SmallVec;

pub struct SFCBlock {
}

pub struct SFCTemplateBlock {

}

pub struct SFCScriptBlock {

}

pub struct SFCStyleBlock {

}

pub struct SFCDescriptor<'a> {
    pub filename: String,
    pub source: &'a str,
    pub template: Option<SFCTemplateBlock>,
    pub scripts: SmallVec<[SFCScriptBlock; 1]>,
    pub styles: SmallVec<[SFCStyleBlock; 1]>,
    pub custom_blocks: Vec<SFCBlock>,
    pub css_vars: Vec<&'a str>,
    /// whether the SFC uses :slotted() modifier.
    /// this is used as a compiler optimization hint.
    pub slotted: bool,
}

pub enum SFCError {
    CompilerError,
    SyntaxError,
}

pub struct SFCParseResult<'a> {
    pub descriptor: SFCDescriptor<'a>,
    pub errors: Vec<SFCError>,
}

pub fn parse_sfc(source: &str) -> SFCParseResult<'_> {
    unimplemented!("TODO")
}
