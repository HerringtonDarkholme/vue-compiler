use compiler::{cast, converter::V_MODEL as V_MODEL_CORE};

use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let mut converted = (V_MODEL_CORE.1)(dir, e, eh);
    use DirectiveConvertResult as DirRet;
    let props = match &mut converted {
        DirRet::Dropped | DirRet::Preserve => return converted,
        DirRet::Converted { value, runtime } => {
            cast!(value, Js::Props)
        }
    };
    let (prop_name, val) = &mut props[0];
    let event_name = match prop_name {
        Js::StrLit(v) => Js::StrLit(*v.clone().be_vmodel()),
        _ => Js::Compound(vec![Js::Src("'onUpdate:' + "), prop_name.clone()]),
    };
    let assignment = generate_assignment();
    props.push((event_name, assignment));
    if need_cache() {
        // TODO, change assignment expr
    }
    converted
}

fn generate_assignment<'a>() -> Js<'a> {
    todo!()
}

fn need_cache() -> bool {
    todo!()
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
