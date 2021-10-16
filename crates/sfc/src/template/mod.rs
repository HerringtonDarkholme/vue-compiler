use compiler::{converter::BaseIR, error::CompilationError};
use compiler::compiler::CompileOption;
mod asset_url;
mod src_set;

use asset_url::{AssetURLOptions, AssetURLTagConfig};

pub enum TransformAssetUrlOption {
    Url(AssetURLOptions),
    Tag(AssetURLTagConfig),
    NoTransform,
}

pub struct SfcTemplateCompileOptions<'a> {
    pub source: &'a str,
    pub filename: &'a str,
    pub id: &'a str,
    pub scoped: bool,
    pub slotted: bool,
    pub is_prod: bool,
    pub ssr: bool,
    pub ssr_css_vars: Vec<&'a str>,
    pub compile_option: CompileOption,
    /// Configure what tags/attributes to transform into asset url imports,
    /// or disable the transform altogether with `false`.
    pub transform_asset_urls: TransformAssetUrlOption,
    // inMap?: RawSourceMap,
    // compiler: TemplateCompiler,
    // preprocessLang?: &'a str
    // preprocessOptions?: any
    // /// In some cases, compiler-sfc may not be inside the project root (e.g. when
    // /// linked or globally installed). In such cases a custom `require` can be
    // /// passed to correctly resolve the preprocessors.
    // // preprocessCustomRequire?: (id: string) => any
}
pub struct SfcTemplateCompileResults<'a> {
    pub code: String,
    pub ast: Option<BaseIR<'a>>,
    pub preamble: Option<String>,
    pub source: String,
    pub tips: Vec<String>,
    pub errors: Vec<CompilationError>,
    // pub map: RawSourceMap
}

pub fn compile_template(_options: SfcTemplateCompileOptions) -> SfcTemplateCompileResults {
    todo!()
}
