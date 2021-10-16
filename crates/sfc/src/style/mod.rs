mod css_module;
mod css_vars;
mod scoped;
use compiler::error::CompilationError;

// pub enum PreprocessLang {
//     Less,
//     Sass,
//     Scss,
//     Styl,
//     Stylus,
// }

pub struct SfcStyleCompileOptions<'a> {
    pub source: &'a str,
    pub filename: &'a str,
    pub id: &'a str,
    pub scoped: bool,
    pub trim: bool,
    pub is_prod: bool,
    // inMap?: RawSourceMap,
    // preprocessLang: Option<PreprocessLang>,
}
pub struct SfcStyleCompileResults<'a> {
    pub code: &'a str,
    pub errors: Vec<CompilationError>,
    // map: RawSourceMap | undefined
    // modules?: Record<string, string>
}

pub fn compile_style<'a>(_source: &'a str, _filename: &'a str) -> SfcStyleCompileResults<'a> {
    todo!()
}
