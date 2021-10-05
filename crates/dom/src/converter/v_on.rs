use compiler::converter::v_on::convert_v_on as convert_v_on_core;

use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};

pub fn convert_v_on<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    use DirectiveConvertResult::Converted;
    let base_converted = convert_v_on_core(dir, e, eh);
    if dir.modifiers.is_empty() {
        return base_converted;
    }
    let mut props = match base_converted {
        Converted {
            value: Js::Props(props),
            ..
        } => props,
        // dropped v-on without expr or v-on="expr"
        other => return other,
    };
    let event_prop = &mut props[0];
    let resolved = resolve_modifiers(&dir.modifiers);
    apply_modifiers(event_prop, resolved);
    Converted {
        value: Js::Props(props),
        runtime: Err(false),
    }
}

struct ResolvedMods<'a> {
    event_option: Vec<&'a str>,
    key_modifiers: Vec<&'a str>,
    non_key_mods: Vec<&'a str>,
}

fn resolve_modifiers<'a>(mods: &[&'a str]) -> ResolvedMods<'a> {
    todo!()
}
fn apply_modifiers<'a>(event: &mut (Js<'a>, Js<'a>), resolved: ResolvedMods<'a>) {
    todo!()
}

pub const V_ON: DirectiveConverter = ("on", convert_v_on);
