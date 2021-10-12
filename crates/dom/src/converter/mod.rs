mod v_html;
mod v_model;
mod v_on;
mod v_show;
mod v_text;

use compiler::converter::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
};
use compiler::ir::JsExpr;

pub const DOM_DIR_CONVERTERS: &[DirectiveConverter] = &[v_on::V_ON, v_model::V_MODEL];
