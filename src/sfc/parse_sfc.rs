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
    filename: String,
    source: &'a str,
    template: Option<SFCTemplateBlock>,
    scripts: SmallVec<[SFCScriptBlock; 1]>,
    styles: SmallVec<[SFCStyleBlock; 1]>,
    custom_blocks: Vec<SFCBlock>,
    css_vars: Vec<&'a str>,
    /// whether the SFC uses :slotted() modifier.
    /// this is used as a compiler optimization hint.
    slotted: bool,
}

enum SFCError {
    CompilerError,
    SyntaxError,
}

pub struct SFCParseResult<'a> {
    descriptor: SFCDescriptor<'a>,
    errors: Vec<SFCError>,
}

pub fn parse_sfc(source: &str) -> SFCParseResult<'_> {
    unimplemented!("TODO")
}
