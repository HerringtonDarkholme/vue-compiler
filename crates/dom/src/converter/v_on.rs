use compiler::converter::v_on::convert_v_on as convert_v_on_core;

use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};
use crate::extension::dom_helper;
use compiler::util::VStr;

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
    let resolved = resolve_modifiers(&dir.modifiers, &event_prop.0);
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

const EVENT_OPTION_MODS: &[&str] = &["passive", "once", "capture"];
const NON_KEY_MODS: &[&str] = &[
    // propagation
    "stop", "prevent", "self", // system modifiers
    "ctrl", "shift", "alt", "meta", "exact", // mouse
    "middle",
];
// mouse click or arrow key
const MAYBE_KEY_MODS: &[&str] = &["left", "right"];
const KEYBOARD_EVENTS: &[&str] = &["keyup", "keydown", "keypress"];

fn resolve_modifiers<'a>(mods: &[&'a str], key: &Js<'a>) -> ResolvedMods<'a> {
    let mut event_option: Vec<&str> = vec![];
    let mut non_key_mods: Vec<&str> = vec![];
    let mut key_modifiers: Vec<&str> = vec![];
    for m in mods {
        if EVENT_OPTION_MODS.contains(m) {
            event_option.push(m);
        } else if MAYBE_KEY_MODS.contains(m) {
            if let Js::StrLit(k) = key {
                let name = k.raw.trim_start_matches("on");
                let is_key_event = KEYBOARD_EVENTS.iter().any(|n| n.eq_ignore_ascii_case(name));
                if is_key_event {
                    key_modifiers.push(m);
                } else {
                    non_key_mods.push(m);
                }
            } else {
                key_modifiers.push(m);
                non_key_mods.push(m);
            }
        } else if NON_KEY_MODS.contains(m) {
            non_key_mods.push(m);
        } else {
            key_modifiers.push(m);
        }
    }
    ResolvedMods {
        event_option,
        non_key_mods,
        key_modifiers,
    }
}
fn apply_modifiers<'a>(event: &mut (Js<'a>, Js<'a>), resolved: ResolvedMods<'a>) {
    let ResolvedMods {
        event_option,
        key_modifiers,
        non_key_mods,
    } = resolved;
    let (key, value) = event;
    if non_key_mods.contains(&"right") {
        *key = convert_click(std::mem::take(key), "contextmenu");
    }
    if non_key_mods.contains(&"middle") {
        *key = convert_click(std::mem::take(key), "mouseup");
    }
    if !non_key_mods.is_empty() {
        let non_keys = non_key_mods.into_iter().map(Js::str_lit).collect();
        *value = Js::Call(
            dom_helper::V_ON_WITH_MODIFIERS,
            vec![std::mem::take(value), Js::Array(non_keys)],
        );
    }
    if !event_option.is_empty() {
        let postfix = event_option
            .into_iter()
            .map(|s| Js::str_lit(*VStr::raw(s).capitalize()))
            .intersperse(Js::Src(" + "));
        let mut new_key_vec = vec![Js::Src("("), std::mem::take(key), Js::Src(")")];
        new_key_vec.extend(postfix);
        *key = Js::Compound(new_key_vec);
    }
}

fn convert_click<'a>(key: Js<'a>, name: &'a str) -> Js<'a> {
    if let Js::StrLit(k) = key {
        Js::str_lit(name)
    } else {
        Js::Compound(vec![
            Js::Src("("),
            key.clone(),
            Js::Src(") === 'onClick' ? "),
            Js::str_lit(name),
            Js::Src(" : ("),
            key,
            Js::Src(")"),
        ])
    }
}

pub const V_ON: DirectiveConverter = ("on", convert_v_on);
