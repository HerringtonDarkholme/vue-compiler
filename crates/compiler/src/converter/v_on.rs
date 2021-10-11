use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
};
use crate::{
    error::CompilationErrorKind as ErrorKind,
    flags::RuntimeHelper,
    ir::{HandlerType, JsExpr as Js},
    parser::DirectiveArg,
    scanner::AttributeValue,
    util::{is_simple_identifier, not_js_identifier, rslint, VStr},
};

// this module process v-on without arg and with arg.
pub fn convert_v_on<'a>(
    dir: &mut Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if let Some(error) = dir.check_empty_expr(ErrorKind::VOnNoExpression) {
        // no argument no expr, just return
        if dir.argument.is_none() {
            eh.on_error(error);
            return DirectiveConvertResult::Dropped;
        }
        // allow @click.stop like
        if dir.modifiers.is_empty() {
            eh.on_error(error);
        }
    }
    let Directive {
        expression,
        argument,
        ..
    } = dir;
    let value = if let Some(arg) = argument {
        let event_name = match arg {
            DirectiveArg::Static(s) => Js::StrLit(*VStr::raw(s).be_handler()),
            DirectiveArg::Dynamic(s) => {
                let e = Js::simple(*s);
                Js::Call(RuntimeHelper::TO_HANDLER_KEY, vec![e])
            }
        };
        let exp = convert_v_on_expr(expression.as_ref());
        Js::Props(vec![(event_name, exp)])
    } else {
        // bare v-on="" does not have mods
        let exp = expression
            .as_ref()
            .expect("v-on with no expr nor arg should be dropped.");
        let exp = Js::simple(exp.content);
        Js::Call(RuntimeHelper::TO_HANDLERS, vec![exp])
    };
    DirectiveConvertResult::Converted {
        value,
        runtime: Err(false),
    }
}

pub fn convert_v_on_expr<'a>(expr: Option<&AttributeValue<'a>>) -> Js<'a> {
    let val = match expr {
        Some(val) => val.content,
        None => return Js::Src("() => {}"),
    };
    Js::func(val)
}

fn is_js_identifier(c: char) -> bool {
    !not_js_identifier(c)
}

// equivalent to this JS regexp
// /^\s*([\w$_]+|(async\s*)?\([^)]*?\))\s*=>|^\s*(async\s+)?function(?:\s+[\w$]+)?\s*\(/
fn is_fn_exp(raw: &str) -> bool {
    // 0. strip whitespace
    let mut raw = raw.trim_start();
    // 1. strip potential async
    raw = raw.trim_start_matches("async ").trim_start();
    // 2.a  async => 123
    if raw.starts_with("=>") {
        return true;
    }
    // 2.b function keyword
    if raw.starts_with("function ") {
        raw = raw.trim_start_matches("function ").trim_start();
        // 3. trim function name
        raw = raw.trim_start_matches(is_js_identifier).trim_start();
        return raw.starts_with('(');
    }
    // 2.c arrow func shorthand. e.g: argName => expr
    if raw.starts_with(is_js_identifier) {
        // 3. strip argName
        raw = raw.trim_start_matches(is_js_identifier).trim_start();
        return raw.starts_with("=>");
    }
    // 2.d arrow func full (arg, arg, ...arg) => expr
    if raw.starts_with('(') {
        raw = raw[1..] // skip (
            .trim_start_matches(|c| c != ')') // skip inside paren
            [1..] // skip )
            .trim_start();
        return raw.starts_with("=>");
    }
    false
}

pub fn is_member_expression(expr: VStr) -> bool {
    if VStr::has_affix(&expr) {
        return false;
    }
    if !expr.raw.starts_with(char::is_alphabetic) {
        return false;
    }
    is_simple_identifier(expr) || rslint::is_member_expression(&expr)
}

pub fn get_handler_type(val: VStr) -> HandlerType {
    if is_member_expression(val) {
        HandlerType::MemberExpr
    } else if is_fn_exp(val.raw) {
        HandlerType::FuncExpr
    } else {
        HandlerType::InlineStmt
    }
}

pub const V_ON: DirectiveConverter = ("on", convert_v_on);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_fn_expr() {
        let positive_cases = [
            "() => 123",
            "([a,b,c]) => 123",
            "(arg) => 123",
            "async => 123",
            "async arg => 123",
            "async (arg) => 123",
            "function (arg) {}",
            "function (arg) {}",
            "async    function   (  arg)  {}",
            "    function   (  arg)  {}",
        ];
        for case in positive_cases {
            assert!(is_fn_exp(case), "{}", case);
        }
        let negative_cases = ["a", "a.b.c", "call()"];
        for case in negative_cases {
            assert!(!is_fn_exp(case), "{}", case);
        }
    }
}
