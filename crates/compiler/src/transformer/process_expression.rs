// 1. track variables introduced in template
// currently only v-for and v-slot
// 2. prefix expression
use super::collect_entities::is_hoisted_asset;
use super::{BaseInfo, CorePassExt, Scope};
use crate::converter::v_on::get_handler_type;
use crate::error::{CompilationError, CompilationErrorKind as ErrorKind, RcErrHandle};
use crate::flags::{RuntimeHelper as RH, StaticLevel};
use crate::ir::JsExpr as Js;
use crate::util::{is_global_allow_listed, is_simple_identifier, rslint, VStr};
use crate::{cast, BindingTypes, SFCInfo, SourceLocation};

pub struct ExpressionProcessor<'a, 'b> {
    pub prefix_identifier: bool,
    pub sfc_info: &'b SFCInfo<'a>,
    pub err_handle: RcErrHandle,
}

impl<'a, 'b> CorePassExt<BaseInfo<'a>, Scope<'a>> for ExpressionProcessor<'a, 'b> {
    fn enter_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {
        self.process_fn_param(p);
        match p {
            Js::Param(id) => shared.add_identifier(id),
            Js::Compound(ids) => {
                for id in only_param_ids(ids) {
                    shared.add_identifier(id);
                }
            }
            _ => panic!("only Js::Param is legal"),
        }
    }
    fn exit_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {
        match p {
            Js::Param(id) => shared.remove_identifier(id),
            Js::Compound(ids) => {
                for id in only_param_ids(ids) {
                    shared.remove_identifier(id);
                }
            }
            _ => panic!("only Js::Param is legal"),
        };
    }
    // only transform expression after its' sub-expression is transformed
    // e.g. compound/array/call expression
    fn exit_js_expr(&mut self, e: &mut Js<'a>, shared: &mut Scope<'a>) {
        self.process_expression(e, shared);
    }
}

impl<'a, 'b> ExpressionProcessor<'a, 'b> {
    // parse expr as function params:
    fn process_fn_param(&self, p: &mut Js) {
        if !self.prefix_identifier {
            return;
        }
        let raw = *cast!(p, Js::Param);
        if is_simple_identifier(VStr::raw(raw)) {
            return;
        }
        // 1. breaks down binding pattern e.g. [a, b, c] => identifiers a, b and c
        // 2. breaks default parameter like v-slot="a = 123" -> (a = 123)
        let broken_atoms = if let Some(atoms) = self.break_down_fn_params(raw) {
            atoms
        } else {
            // TODO: add identifier location
            self.report_wrong_identifier(SourceLocation::default());
            return;
        };
        // 3. reunite these 1 and 2 to a compound expression
        *p = reunite_atoms(raw, broken_atoms, |atom| {
            let is_param = atom.property;
            let text = &raw[atom.range];
            if is_param {
                Js::Param(text)
            } else {
                Js::Simple(VStr::raw(text), StaticLevel::NotStatic)
            }
        })
    }
    fn process_expression(&self, e: &mut Js<'a>, scope: &mut Scope) {
        if !self.prefix_identifier {
            return;
        }
        // hoisted component/directive does not need prefixing
        if is_hoisted_asset(e).is_some() {
            return;
        }
        use crate::ir::HandlerType::InlineStmt;
        // complex expr will be handled recursively in transformer
        let (exp, mut mock_js) = match e {
            Js::FuncSimple(v, l) => (*v, Js::Simple(*v, *l)),
            Js::Simple(..) => return self.process_simple_expr(e, scope),
            _ => return,
        };
        let ty = get_handler_type(exp);
        if matches!(ty, InlineStmt) {
            scope.add_identifier("$event");
        }
        self.process_simple_expr(&mut mock_js, scope);
        *e = match mock_js {
            Js::Simple(s, l) => Js::FuncSimple(s, l),
            Js::Compound(v) => Js::FuncCompound(v.into_boxed_slice()),
            _ => panic!("impossible"),
        };
        if matches!(ty, InlineStmt) {
            scope.remove_identifier("$event");
        }
    }

    fn process_simple_expr(&self, e: &mut Js<'a>, scope: &Scope) {
        if self.process_expr_fast(e, scope) {
            return;
        }
        self.process_with_js_parser(e, scope)
    }

    /// prefix _ctx without parsing JS
    fn process_expr_fast(&self, e: &mut Js<'a>, scope: &Scope) -> bool {
        let (v, level) = match e {
            Js::Simple(v, level) => (v, level),
            _ => panic!("impossible"),
        };
        if !is_simple_identifier(*v) {
            return false;
        }
        let raw_exp = v.raw;
        let is_scope_reference = scope.has_identifier(raw_exp);
        let is_allowed_global = is_global_allow_listed(raw_exp);
        let is_literal = matches!(raw_exp, "true" | "false" | "null" | "this");
        if !is_scope_reference && !is_allowed_global && !is_literal {
            // const bindings from setup can skip patching but cannot be hoisted
            // NB: this only applies to simple expression. e.g :prop="constBind()"
            let bindings = &self.sfc_info.binding_metadata;
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

    fn process_with_js_parser(&self, e: &mut Js<'a>, scope: &Scope) {
        let (v, level) = match e {
            Js::Simple(v, level) => (v, level),
            _ => panic!("impossible"),
        };
        let raw = v.raw;
        let broken = self.break_down_complex_expression(raw, scope);
        let (broken_atoms, local_ref) = if let Some(pair) = broken {
            pair
        } else {
            // TODO: add identifier location
            self.report_wrong_identifier(SourceLocation::default());
            return;
        };
        // no prefixed identifier found
        if broken_atoms.is_empty() {
            // if expr has no template var nor prefixed var, it can be hoisted as static
            // NOTE: func call and member access must be bailed for potential side-effect
            let side_effect = raw.contains('(') || raw.contains('.');
            *level = if !local_ref && !side_effect {
                StaticLevel::CanStringify
            } else {
                StaticLevel::NotStatic
            };
            return;
        }
        *e = reunite_atoms(raw, broken_atoms, |atom| {
            let prop = atom.property;
            let id_str = VStr::raw(&raw[atom.range]);
            let rewritten = self.rewrite_identifier(id_str, StaticLevel::NotStatic, prop.ctx_type);
            if prop.is_obj_shorthand {
                Js::Compound(vec![Js::StrLit(id_str), Js::Src(": "), rewritten])
            } else {
                rewritten
            }
        });
    }
    fn rewrite_identifier(&self, raw: VStr<'a>, level: StaticLevel, ctx: CtxType<'a>) -> Js<'a> {
        let binding = self.sfc_info.binding_metadata.get(&raw.raw);
        if let Some(bind) = binding {
            if self.sfc_info.inline {
                rewrite_inline_identifier(raw, level, bind, ctx)
            } else {
                bind.get_js_prop(raw, level)
            }
        } else {
            debug_assert!(level == StaticLevel::NotStatic);
            Js::simple(*raw.clone().prefix_ctx())
        }
    }
    fn report_wrong_identifier(&self, loc: SourceLocation) {
        let error = CompilationError::new(ErrorKind::InvalidExpression).with_location(loc);
        self.err_handle.on_error(error);
    }

    fn break_down_complex_expression(
        &self,
        raw: &'a str,
        scope: &Scope,
    ) -> Option<(FreeVarAtoms<'a>, bool)> {
        let expr = rslint::parse_js_expr(raw)?;
        let inline = self.sfc_info.inline;
        let mut atoms = vec![];
        let mut has_local_ref = false;
        rslint::walk_free_variables(expr, |fv| {
            let id_text = fv.text();
            // skip global variable prefixing
            if is_global_allow_listed(&id_text) || id_text == "require" {
                return;
            }
            let range = fv.range();
            // skip id defined in the template scope
            if scope.has_identifier(&raw[range.clone()]) {
                has_local_ref = true;
                return;
            }
            let ctx_type = if inline { todo!() } else { CtxType::NoWrite };
            atoms.push(Atom {
                range,
                property: FreeVarProp {
                    ctx_type,
                    is_obj_shorthand: fv.is_shorthand(),
                },
            })
        });
        atoms.sort_by_key(|r| r.range.start);
        Some((atoms, has_local_ref))
    }

    /// Atom's property records if it is param identifier
    fn break_down_fn_params(&self, raw: &'a str) -> Option<Vec<Atom<bool>>> {
        let param = rslint::parse_fn_param(raw)?;
        // range is offset by -1 due to the wrapping parens when parsed
        let offset = if raw.starts_with('(') { 0 } else { 1 };
        let mut atoms = vec![];
        rslint::walk_param_and_default_arg(param, |range, is_param| {
            atoms.push(Atom {
                range: range.start - offset..range.end - offset,
                property: is_param,
            });
        });
        atoms.sort_by_key(|r| r.range.start);
        Some(atoms)
    }
}

// This implementation assumes that broken param expression has only two kinds subexpr:
// 1. param identifiers represented by Js::Param
// 2. expression in default binding that has been prefixed
fn only_param_ids<'a, 'b>(ids: &'b [Js<'a>]) -> impl Iterator<Item = &'a str> + 'b {
    ids.iter().filter_map(|id| match id {
        Js::Param(p) => Some(*p),
        Js::Src(_) => None,
        Js::Simple(..) => None,
        Js::Compound(..) => None, // object shorthand
        _ => panic!("Illegal sub expr kind in param."),
    })
}

/// Atom is the atomic identifier text range in the expression.
/// Property is the additional information for rewriting.
struct Atom<T> {
    range: std::ops::Range<usize>,
    property: T,
}

struct FreeVarProp<'a> {
    is_obj_shorthand: bool,
    ctx_type: CtxType<'a>,
}
type FreeVarAtoms<'a> = Vec<Atom<FreeVarProp<'a>>>;

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

fn reunite_atoms<'a, T, F>(raw: &'a str, atoms: Vec<Atom<T>>, mut rewrite: F) -> Js<'a>
where
    F: FnMut(Atom<T>) -> Js<'a>,
{
    // expr without atoms have specific processing outside
    debug_assert!(!atoms.is_empty());
    // the only one ident that spans the text should be handled in fast path
    debug_assert!(atoms.len() > 1 || atoms[0].range.len() < raw.len());
    let mut inner = vec![];
    let mut last = 0;
    for atom in atoms {
        let range = &atom.range;
        if last < range.start {
            let comp = Js::Src(&raw[last..range.start]);
            inner.push(comp);
        }
        last = range.end;
        let rewritten = rewrite(atom);
        inner.push(rewritten);
    }
    if last < raw.len() {
        inner.push(Js::Src(&raw[last..]));
    }
    Js::Compound(inner)
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
    if VStr::is_event_assign(&raw) {
        todo!("handle event assign differently")
    }
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

#[cfg(test)]
mod test {
    use super::super::{
        test::{base_convert, transformer_ext},
        BaseRoot, Transformer,
    };
    use super::*;
    use crate::cast;
    use crate::converter::BaseIR;
    use crate::error::{NoopErrorHandler, RcErrHandle, VecErrorHandler};
    use crate::ir::IRNode;
    use std::rc::Rc;

    fn transform_with_err(s: &str, handler: RcErrHandle) -> BaseRoot {
        let mut ir = base_convert(s);
        let exp = ExpressionProcessor {
            prefix_identifier: true,
            sfc_info: &Default::default(),
            err_handle: handler,
        };
        let mut transformer = transformer_ext(exp);
        transformer.transform(&mut ir);
        ir
    }

    fn transform(s: &str) -> BaseRoot {
        transform_with_err(s, Rc::new(NoopErrorHandler))
    }
    fn first_child(ir: BaseRoot) -> BaseIR {
        ir.body.into_iter().next().unwrap()
    }

    #[test]
    fn test_interpolation_prefix() {
        let ir = transform("{{test}}");
        let text = cast!(first_child(ir), IRNode::TextCall);
        let text = match &text.texts[0] {
            Js::Call(_, r) => &r[0],
            _ => panic!("wrong interpolation"),
        };
        let expr = cast!(text, Js::Simple);
        assert_eq!(expr.into_string(), "_ctx.test");
    }
    #[test]
    fn test_prop_prefix() {
        let ir = transform("<p :test='a'/>");
        let vn = cast!(first_child(ir), IRNode::VNodeCall);
        let props = vn.props.unwrap();
        let props = cast!(props, Js::Props);
        let key = cast!(&props[0].0, Js::StrLit);
        assert_eq!(key.into_string(), "test");
        let expr = cast!(&props[0].1, Js::Simple);
        assert_eq!(expr.into_string(), "_ctx.a");
    }
    #[test]
    fn test_v_bind_prefix() {
        let ir = transform("<p v-bind='b'/>");
        let vn = cast!(&ir.body[0], IRNode::VNodeCall);
        let props = vn.props.as_ref().unwrap();
        let expr = cast!(props, Js::Simple);
        assert_eq!(expr.into_string(), "_ctx.b");
    }
    #[test]
    fn test_prefix_v_for() {
        let ir = transform("<p v-for='a in b'/>");
        let v_for = cast!(first_child(ir), IRNode::For);
        let b = cast!(v_for.source, Js::Simple);
        let a = cast!(v_for.parse_result.value, Js::Param);
        assert_eq!(a, "a");
        assert_eq!(b.into_string(), "_ctx.b");
    }
    #[test]
    fn test_complex_expression() {
        let ir = transform("{{a + b}}");
        let text = cast!(first_child(ir), IRNode::TextCall);
        let text = match &text.texts[0] {
            Js::Call(_, r) => &r[0],
            _ => panic!("wrong interpolation"),
        };
        let expr = cast!(text, Js::Compound);
        let a = cast!(expr[0], Js::Simple);
        let b = cast!(expr[2], Js::Simple);
        assert_eq!(a.into_string(), "_ctx.a");
        assert_eq!(b.into_string(), "_ctx.b");
    }

    #[test]
    fn test_transform_shorthand() {
        let ir = transform("{{ {a} }}");
        let text = cast!(first_child(ir), IRNode::TextCall);
        let text = match &text.texts[0] {
            Js::Call(_, r) => &r[0],
            _ => panic!("wrong interpolation"),
        };
        let expr = cast!(text, Js::Compound);
        let prop = cast!(&expr[1], Js::Compound);
        let key = cast!(prop[0], Js::StrLit);
        let colon = cast!(prop[1], Js::Src);
        let val = cast!(prop[2], Js::Simple);
        assert_eq!(key.into_string(), "a");
        assert_eq!(colon, ": ");
        assert_eq!(val.into_string(), "_ctx.a");
    }

    #[test]
    fn test_transform_fn_param() {
        let ir = transform("<p v-for='a=c in b'/>");
        let v_for = cast!(first_child(ir), IRNode::For);
        let val = cast!(v_for.parse_result.value, Js::Compound);
        let a = cast!(val[0], Js::Param);
        let c = cast!(val[2], Js::Simple);
        assert_eq!(a, "a");
        assert_eq!(c.into_string(), "_ctx.c");
    }
    #[test]
    fn test_transform_destruct() {
        let ir = transform("<p v-for='{a: dd} in b' :yes='a' :not='dd' />");
        let v_for = cast!(first_child(ir), IRNode::For);
        let val = cast!(v_for.parse_result.value, Js::Compound);
        let dd = cast!(val[1], Js::Param);
        assert_eq!(dd, "dd");
        let p = cast!(*v_for.child, IRNode::VNodeCall);
        let props = cast!(p.props.unwrap(), Js::Props);
        let a = cast!(props[0].1, Js::Simple);
        let dd = cast!(props[1].1, Js::Simple);
        assert_eq!(a.into_string(), "_ctx.a");
        assert_eq!(dd.into_string(), "dd");
    }

    #[test]
    fn test_transform_default_shorthand() {
        let ir = transform("<p v-for='a={c} in b'/>");
        let v_for = cast!(first_child(ir), IRNode::For);
        let val = cast!(v_for.parse_result.value, Js::Compound);
        let c = cast!(&val[2], Js::Compound);
        let prop = cast!(&c[1], Js::Compound);
        let key = cast!(prop[0], Js::StrLit);
        let val = cast!(prop[2], Js::Simple);
        assert_eq!(key.into_string(), "c");
        assert_eq!(val.into_string(), "_ctx.c");
    }

    #[test]
    fn test_error_expression() {
        let error_handler = Rc::new(VecErrorHandler::default());
        transform_with_err("{{ +invalid+ }}", error_handler.clone());
        let errs = error_handler.errors();
        assert!(!errs.is_empty());
        let kind = &errs[0].kind;
        assert!(matches!(kind, ErrorKind::InvalidExpression));
    }
}
