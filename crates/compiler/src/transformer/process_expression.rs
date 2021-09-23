// 1. track variables introduced in template
// currently only v-for and v-slot
// 2. prefix expression
use super::{BaseInfo, CorePassExt, Scope, TransformOption};
use crate::converter::{BaseRoot, BindingTypes, JsExpr as Js};
use crate::flags::{RuntimeHelper as RH, StaticLevel};
use crate::util::{is_global_allow_listed, is_simple_identifier, VStr};

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
    // only transform expression after its' sub-expression is transformed
    // e.g. compound/array/call expression
    fn exit_js_expr(&mut self, e: &mut Js<'a>, shared: &mut Scope<'a>) {
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
            // const bindings from setup can skip patching but cannot be hoisted
            // NB: this only applies to simple expression. e.g :prop="constBind()"
            let bindings = &self.option.binding_metadata;
            let lvl = match bindings.get(raw_exp) {
                Some(BindingTypes::SetupConst) => StaticLevel::CanSkipPatch,
                _ => *level,
            };
            *e = self.rewrite_identifier(*v, lvl, CtxType::NoWrite);
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
    fn rewrite_identifier<'a>(
        &self,
        raw: VStr<'a>,
        level: StaticLevel,
        ctx: CtxType<'a>,
    ) -> Js<'a> {
        let binding = self.option.binding_metadata.get(&raw.raw);
        if let Some(bind) = binding {
            if self.option.inline {
                rewrite_inline_identifier(raw, level, bind, ctx)
            } else {
                bind.get_js_prop(raw, level)
            }
        } else {
            debug_assert!(level == StaticLevel::NotStatic);
            let prop = Js::simple(raw);
            Js::Compound(vec![Js::Src("$ctx."), prop])
        }
    }
}

enum CtxType<'a> {
    /// ref = value, ref += value
    Assign(Js<'a>),
    /// ref++, ++ref, ...
    Update(bool, Js<'a>),
    /// ({x}) = y
    Destructure,
    /// No reactive var writing
    NoWrite,
}

// parse expr as function params:
// 1. breaks down binding pattern e.g. [a, b, c] => identifiers a, b and c
// 2. patch default parameter like v-slot="a = 123" -> (a = 123)
fn process_fn_param(p: &mut Js) {
    todo!()
}

fn rewrite_inline_identifier<'a>(
    raw: VStr<'a>,
    level: StaticLevel,
    bind: &BindingTypes,
    ctx: CtxType<'a>,
) -> Js<'a> {
    use BindingTypes as BT;
    debug_assert!(level == StaticLevel::NotStatic || bind == &BT::SetupConst);
    let expr = move || Js::Simple(raw, level);
    let dot_value = Js::Compound(vec![expr(), Js::Src(".value")]);
    match bind {
        BT::SetupConst => expr(),
        BT::SetupRef => dot_value,
        BT::SetupMaybeRef => {
            // const binding that may or may not be ref
            // if it's not a ref, then assignments don't make sense -
            // so we ignore the non-ref assignment case and generate code
            // that assumes the value to be a ref for more efficiency
            if !matches!(ctx, CtxType::NoWrite) {
                dot_value
            } else {
                Js::Call(RH::Unref, vec![expr()])
            }
        }
        BT::SetupLet => rewrite_setup_let(ctx, expr, dot_value),
        BT::Props => Js::Compound(vec![Js::Src("__props."), expr()]),
        BT::Data | BT::Options => Js::Compound(vec![Js::Src("_ctx."), expr()]),
    }
}

fn rewrite_setup_let<'a, E>(ctx: CtxType<'a>, expr: E, dot_value: Js<'a>) -> Js<'a>
where
    E: Fn() -> Js<'a>,
{
    match ctx {
        CtxType::Assign(assign) => Js::Compound(vec![
            Js::Call(RH::IsRef, vec![expr()]),
            Js::Src("? "),
            dot_value,
            assign.clone(),
            Js::Src(": "),
            expr(),
            assign,
        ]),
        CtxType::Update(is_pre, op) => {
            let mut v = vec![Js::Call(RH::IsRef, vec![expr()])];
            v.push(Js::Src("? "));
            let push = |v: &mut Vec<_>, val, op| {
                if is_pre {
                    v.extend([op, val]);
                } else {
                    v.extend([val, op]);
                }
            };
            push(&mut v, dot_value, op.clone());
            v.push(Js::Src(": "));
            push(&mut v, expr(), op);
            Js::Compound(v)
        }
        CtxType::Destructure => {
            // TODO let binding in a destructure assignment - it's very tricky to
            // handle both possible cases here without altering the original
            // structure of the code, so we just assume it's not a ref here for now
            expr()
        }
        CtxType::NoWrite => Js::Call(RH::Unref, vec![expr()]),
    }
}
