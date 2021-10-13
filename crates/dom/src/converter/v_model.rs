use compiler::cast;
use compiler::converter::{CompilationError, v_model::convert_v_model_event};
use compiler::flags::RuntimeHelper;
use compiler::ir::JsExpr as Js;
use compiler::parser::{ElemProp, DirectiveArg};
use compiler::util::find_prop;
use super::DirectiveConvertResult;
use crate::extension::{dom_helper as dh, DomError};
use crate::options::is_native_tag;

use super::{CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if e.is_component() {
        return convert_v_model_event(dir, e, eh);
    }
    if dir.argument.is_some() {
        let error = CompilationError::extended(DomError::VModelArgOnElement)
            .with_location(dir.location.clone());
        eh.on_error(error);
    }
    let mut base = convert_v_model_event(dir, e, eh);
    let (value, runtime) = match &mut base {
        DirectiveConvertResult::Dropped | DirectiveConvertResult::Preserve => return base,
        DirectiveConvertResult::Converted { value, runtime } => (value, runtime),
    };
    let runtime_to_use = match compute_v_model_runtime(e, dir) {
        Ok(rt) => {
            if matches!(rt, dh::V_MODEL_TEXT | dh::V_MODEL_SELECT) {
                check_redundant_value_prop(e, eh);
            }
            *runtime = Ok(rt);
        }
        Err(error) => eh.on_error(error),
    };
    // native vmodel doesn't need the `modelValue` props since they are also
    // passed to the runtime as `binding.value`. removing it reduces code size.
    let props = cast!(value, Js::Props);
    for i in 0..props.len() {
        if let Js::StrLit(s) = props[i].0 {
            if s.raw == "modelValue" {
                props.remove(i);
                break;
            }
        }
    }
    base
}

fn check_redundant_value_prop(e: &Element, eh: &dyn ErrorHandler) {
    if let Some(prop) = find_prop(e, "value") {
        let loc = prop.get_ref().get_location();
        let error =
            CompilationError::extended(DomError::VModelUnnecessaryValue).with_location(loc.clone());
        eh.on_error(error);
    }
}

type RuntimeResult = Result<RuntimeHelper, CompilationError>;
fn compute_v_model_runtime(e: &Element, dir: &Directive) -> RuntimeResult {
    let tag = e.tag_name;
    // tag is not component nor native, so it must be custom
    let is_custom_element = !is_native_tag(tag);
    if !["input", "select", "textarea"].contains(&tag) && !is_custom_element {
        let error = CompilationError::extended(DomError::VModelOnInvalidElement)
            .with_location(dir.location.clone());
        return Err(error);
    }
    if tag == "select" {
        return Ok(dh::V_MODEL_SELECT);
    } else if tag == "textarea" {
        // text area
        return Ok(dh::V_MODEL_TEXT);
    }
    debug_assert!(tag == "input" || is_custom_element);
    // input or custom_element
    let ty = match find_prop(e, "type") {
        Some(ty) => ty,
        None if has_dynamic_v_bind(e) => return Ok(dh::V_MODEL_DYNAMIC),
        None => return Ok(dh::V_MODEL_TEXT),
    };
    let ty = ty.get_ref();
    let val = match ty {
        ElemProp::Dir(..) => return Ok(dh::V_MODEL_DYNAMIC),
        ElemProp::Attr(attr) => attr.value.as_ref().expect("non empty"),
    };
    match val.content.raw {
        "radio" => Ok(dh::V_MODEL_RADIO),
        "checkbox" => Ok(dh::V_MODEL_CHECKBOX),
        "file" => {
            let error = CompilationError::extended(DomError::VModelOnFileInputElement)
                .with_location(val.location.clone());
            Err(error)
        }
        _ => Ok(dh::V_MODEL_TEXT),
    }
}

pub fn has_dynamic_v_bind(e: &Element) -> bool {
    e.properties
        .iter()
        .filter_map(|prop| match prop {
            ElemProp::Dir(v) => Some(v),
            ElemProp::Attr(_) => None,
        })
        .filter(|d| d.name == "bind")
        .any(|d| {
            d.argument
                .as_ref()
                .map_or(true, |arg| matches!(arg, DirectiveArg::Dynamic(..)))
        })
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
