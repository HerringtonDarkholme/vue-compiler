pub type TemplateDescriptor = String;
pub type ScriptDescriptor = String;
pub type StyleDescriptor = String;

pub struct SFCDescriptor<'a> {
    filename: String,
    source: &'a str,
    template: TemplateDescriptor,
    script: ScriptDescriptor,
    style: StyleDescriptor,
}

/*
  template: SFCTemplateBlock | null
  script: SFCScriptBlock | null
  scriptSetup: SFCScriptBlock | null
  styles: SFCStyleBlock[]
  customBlocks: SFCBlock[]
  cssVars: string[]
  // whether the SFC uses :slotted() modifier.
  // this is used as a compiler optimization hint.
  slotted: boolean
}

export interface SFCParseResult {
  descriptor: SFCDescriptor
  errors: (CompilerError | SyntaxError)[]
}
 * */

pub fn parse_sfc(source: String) -> SFCDescriptor {
    unimplemented!("TODO")
}
