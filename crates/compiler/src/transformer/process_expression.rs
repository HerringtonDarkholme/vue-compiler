// 1. track variables introduced in template
// currently only v-for and v-slot
// 2. prefix expression
use super::{BaseInfo, CorePassExt, TransformOption};
use crate::converter::{BaseRoot, BindingTypes, JsExpr as Js};
use crate::flags::StaticLevel;
use crate::util::{is_global_allow_listed, is_simple_identifier, VStr};
use rustc_hash::FxHashMap;

pub struct Scope<'a> {
    identifiers: FxHashMap<VStr<'a>, usize>,
}

pub struct ExpressionProcessor<'b> {
    option: &'b TransformOption,
}

impl<'a, 'b> CorePassExt<BaseInfo<'a>, Scope<'a>> for ExpressionProcessor<'b> {
    fn enter_root(&mut self, r: &mut BaseRoot<'a>, shared: &mut Scope<'a>) {}
    fn exit_root(&mut self, r: &mut BaseRoot<'a>, shared: &mut Scope<'a>) {}
    fn enter_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {
        process_fn_param(p);
        let a = match p {
            Js::Simple(v, _) => *v,
            Js::Compound(_) => todo!(),
            _ => panic!("param should only be expression"),
        };
        *shared.identifiers.entry(a).or_default() += 1;
    }
    fn exit_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {
        let a = match p {
            Js::Simple(v, _) => *v,
            Js::Compound(_) => todo!(),
            _ => panic!("param should only be expression"),
        };
        *shared.identifiers.entry(a).or_default() -= 1;
    }
    fn enter_js_expr(&mut self, e: &mut Js<'a>, shared: &mut Scope<'a>) {
        self.process_expression(e, shared);
    }
}

impl<'b> ExpressionProcessor<'b> {
    fn process_expression(&self, e: &mut Js, scope: &Scope) {
        if !self.option.prefix_identifier {
            return;
        }
        if self.process_expr_fast(e, scope) {
            return;
        }
        self.process_with_swc(e);
    }

    /// prefix _ctx without ast parsing
    fn process_expr_fast(&self, e: &mut Js, scope: &Scope) -> bool {
        let (v, level) = match e {
            Js::Simple(v, level) => (v, level),
            _ => return false,
        };
        if !is_simple_identifier(*v) {
            return false;
        }
        let raw_exp = v.raw;
        let is_scope_reference = scope.identifiers.contains_key(v);
        let is_allowed_global = is_global_allow_listed(raw_exp);
        let is_literal = matches!(raw_exp, "true" | "false" | "null" | "this");
        if !is_scope_reference && !is_allowed_global && !is_literal {
            // const bindings exposed from setup can skip patching
            // but cannot be hoisted to module scope
            let bindings = &self.option.binding_metadata;
            let lvl = match bindings.get(raw_exp) {
                Some(BindingTypes::SetupConst) => StaticLevel::CanSkipPatch,
                _ => *level,
            };
            *e = self.rewrite_identifier(*v, lvl);
        } else if !is_scope_reference {
            *level = if is_literal {
                StaticLevel::CanStringify
            } else {
                StaticLevel::CanHoist
            };
        }
        true
    }

    fn process_with_swc(&self, e: &mut Js) {
        todo!()
    }
    fn rewrite_identifier<'a>(&self, raw: VStr<'a>, static_level: StaticLevel) -> Js<'a> {
        let binding = self.option.binding_metadata.get(&raw.raw);
        if self.option.inline {
            return rewrite_inline_identifier(raw);
        }
        if let Some(bind) = binding {
            bind.get_js_prop(raw)
        } else {
            let prop = Js::Simple(raw, static_level);
            Js::Compound(vec![Js::Src("$ctx."), prop])
        }
    }
}

// parse expr as function params:
// 1. breaks down binding pattern e.g. [a, b, c] => identifiers a, b and c
// 2. patch default parameter like v-slot="a = 123" -> (a = 123)
fn process_fn_param(p: &mut Js) {
    todo!()
}

fn rewrite_inline_identifier(raw: VStr) -> Js {
    todo!()
}
