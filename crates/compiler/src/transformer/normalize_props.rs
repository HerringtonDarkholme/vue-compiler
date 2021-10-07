/// extract class/style for faster runtime patching
use crate::ir::JsExpr as Js;

pub fn pre_normalize_prop(prop_expr: Option<Js>) -> Option<Js> {
    todo!("pre-normalize props only in DOM for now. usable in any platform")
}
