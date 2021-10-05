use crate::flags::StaticLevel;
use crate::parser::ElementType;
use crate::{
    cast,
    error::{CompilationError as Error, CompilationErrorKind as ErrorKind},
    ir::{JsExpr as Js, Prop},
    parser::DirectiveArg,
    util::VStr,
};

use super::{
    v_on::is_member_expression, CoreDirConvRet, Directive, DirectiveConvertResult,
    DirectiveConverter, Element, ErrorHandler,
};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    element: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if let Some(error) = dir.check_empty_expr(ErrorKind::VModelNoExpression) {
        eh.on_error(error);
        return DirectiveConvertResult::Dropped;
    }
    let Directive {
        expression,
        modifiers,
        argument,
        head_loc,
        ..
    } = dir;
    let attr_value = expression.take().expect("empty dir should be dropped");
    let val = attr_value.content;
    // TODO: looks like pattern can also work?
    if !is_member_expression(val) {
        let error =
            Error::new(ErrorKind::VModelMalformedExpression).with_location(attr_value.location);
        eh.on_error(error);
        return DirectiveConvertResult::Dropped;
    }
    // TODO: add scope variable check

    let prop_name = if let Some(arg) = argument {
        match arg {
            DirectiveArg::Static(s) => Js::str_lit(*s),
            DirectiveArg::Dynamic(d) => Js::simple(*d),
        }
    } else {
        Js::str_lit("modelValue")
    };
    let mut props = vec![(prop_name, Js::Simple(val, StaticLevel::NotStatic))];
    if let Some(mods) = component_mods_prop(dir, element) {
        props.push(mods);
    }
    DirectiveConvertResult::Converted {
        value: Js::Props(props),
        runtime: Err(false),
    }
}

fn component_mods_prop<'a>(dir: &Directive<'a>, elem: &Element<'a>) -> Option<Prop<'a>> {
    let Directive {
        argument,
        modifiers,
        ..
    } = dir;
    // only v-model on component need compile modifiers in the props
    // native inputs have v-model inside the children
    if modifiers.is_empty() || elem.tag_type != ElementType::Component {
        return None;
    }
    let modifiers_key = if let Some(arg) = argument {
        match arg {
            DirectiveArg::Static(s) => Js::StrLit(*VStr::raw(s).suffix_mod()),
            DirectiveArg::Dynamic(d) => {
                Js::Compound(vec![Js::simple(*d), Js::Src(" + 'Modifiers'")])
            }
        }
    } else {
        Js::str_lit("modelModifiers")
    };
    let mod_value = modifiers
        .iter()
        .map(|s| (Js::str_lit(*s), Js::Src("true")))
        .collect();
    Some((modifiers_key, Js::Props(mod_value)))
}

pub fn convert_v_model_event(converted: &mut DirectiveConvertResult<Js>) {
    use DirectiveConvertResult as DirRet;
    let props = match converted {
        DirRet::Dropped | DirRet::Preserve => return,
        DirRet::Converted { value, runtime } => {
            cast!(value, Js::Props)
        }
    };
    let (prop_name, val) = &mut props[0];
    let event_name = match prop_name {
        Js::StrLit(v) => Js::StrLit(*v.clone().be_vmodel()),
        _ => Js::Compound(vec![Js::Src("'onUpdate:' + "), prop_name.clone()]),
    };
    let val_expr = *cast!(val, Js::Simple).clone().assign_event();
    let assignment = Js::func(val_expr);
    // TODO, cache assignment expr
    props.push((event_name, assignment));
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
