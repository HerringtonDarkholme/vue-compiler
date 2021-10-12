use compiler::{
    Namespace, codegen::ScriptMode, compiler::CompileOption, converter::RcErrHandle,
    flags::RuntimeHelper, parser::Element, scanner::TextMode,
};
use crate::{converter::DOM_DIR_CONVERTERS, extension::dom_helper};
use phf::{phf_set, Set};

const NATIVE_TAGS: Set<&str> = phf_set! {
    // HTML_TAGS
    "html","body","base","head","link","meta","style","title","address","article","aside","footer",
    "header","h1","h2","h3","h4","h5","h6","nav","section","div","dd","dl","dt","figcaption", "figure",
    "picture","hr","img","li","main","ol","p","pre","ul","a","b","abbr","bdi","bdo","br","cite","code",
    "data","dfn","em","i","kbd","mark","q","rp","rt","ruby","s","samp","small","span","strong","sub","sup",
    "time","u","var","wbr","area","audio","map","track","video","embed","object","param","source",
    "canvas","script","noscript","del","ins","caption","col","colgroup","table","thead","tbody","td",
    "th","tr","button","datalist","fieldset","form","input","label","legend","meter","optgroup",
    "option","output","progress","select","textarea","details","dialog","menu",
    "summary","template","blockquote","iframe","tfoot",
    // SVG_TAGS
    "svg","animate","animateMotion","animateTransform","circle","clipPath","color-profile",
    "defs","desc","discard","ellipse","feBlend","feColorMatrix","feComponentTransfer",
    "feComposite","feConvolveMatrix","feDiffuseLighting","feDisplacementMap",
    "feDistanceLight","feDropShadow","feFlood","feFuncA","feFuncB","feFuncG","feFuncR",
    "feGaussianBlur","feImage","feMerge","feMergeNode","feMorphology","feOffset",
    "fePointLight","feSpecularLighting","feSpotLight","feTile","feTurbulence","filter",
    "foreignObject","g","hatch","hatchpath","image","line","linearGradient","marker","mask",
    "mesh","meshgradient","meshpatch","meshrow","metadata","mpath","path","pattern",
    "polygon","polyline","radialGradient","rect","set","solidcolor","stop","switch","symbol",
    "text","textPath","tspan","unknown","use","view", // "title"
};

fn is_native_tag(s: &str) -> bool {
    NATIVE_TAGS.contains(s)
}
fn is_pre_tag(s: &str) -> bool {
    s.eq_ignore_ascii_case("pre")
}

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];
fn is_void_tag(s: &str) -> bool {
    VOID_TAGS.contains(&s)
}

fn get_builtin_component(s: &str) -> Option<RuntimeHelper> {
    todo!()
}

fn get_text_mode(s: &str) -> TextMode {
    match s {
        "style" | "script" | "iframe" | "noscript" => TextMode::RawText,
        "textarea" | "title" => TextMode::RcData,
        _ => TextMode::Data,
    }
}

fn get_namespace(s: &str, ancestors: &[Element]) -> Namespace {
    todo!()
}

pub fn compile_option(error_handler: RcErrHandle) -> CompileOption {
    CompileOption {
        is_native_tag,
        get_text_mode,
        is_pre_tag,
        is_void_tag,
        get_builtin_component,
        get_namespace,
        delimiters: ("{{".to_string(), "}}".to_string()),
        directive_converters: DOM_DIR_CONVERTERS.iter().copied().collect(),
        helper_strs: dom_helper::DOM_HELPER_MAP,
        error_handler,
        mode: ScriptMode::Function {
            prefix_identifier: false,
            runtime_global_name: "Vue".into(),
        },
        ..Default::default()
    }
}
