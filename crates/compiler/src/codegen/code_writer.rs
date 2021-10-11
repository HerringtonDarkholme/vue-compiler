use super::{CodeGenerateOption, ScriptMode, CoreCodeGenerator};
use crate::flags::{HelperCollector, PatchFlag, RuntimeHelper as RH, SlotFlag};
use crate::converter::v_on::get_handler_type;
use crate::converter::{BaseConvertInfo, BaseIR, BaseRoot, TopScope};
use crate::transformer::{
    BaseFor, BaseIf, BaseRenderSlot, BaseSlotFn, BaseText, BaseVNode, BaseVSlot, BaseCache,
};
use crate::ir::{self as C, IRNode, JsExpr as Js, RenderSlotIR, RuntimeDir, VNodeIR, HandlerType};
use crate::util::{get_vnode_call_helper, is_simple_identifier, VStr};
use crate::SFCInfo;

use smallvec::SmallVec;
use std::{
    fmt::{self, Write},
    io::{self, Write as ioWrite},
    rc::Rc,
    iter,
};

type Output = fmt::Result;

pub struct WriteAdaptor<T: ioWrite> {
    inner: T,
    io_error: Option<io::Error>,
}
impl<T: ioWrite> WriteAdaptor<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            io_error: None,
        }
    }
    pub fn get_io_error(&mut self) -> io::Error {
        self.io_error
            .take()
            .unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "unexpected fmt error"))
    }
}

impl<T: ioWrite> fmt::Write for WriteAdaptor<T> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> Output {
        match self.inner.write_all(s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.io_error = Some(err);
                Err(fmt::Error)
            }
        }
    }
}

pub struct CodeWriter<'a, T: ioWrite> {
    pub writer: WriteAdaptor<T>,
    option: Rc<CodeGenerateOption>,
    sfc_info: &'a SFCInfo<'a>,
    indent_level: usize,
    closing_brackets: usize,
    cache_count: usize,
    in_alterable: bool,
    helpers: HelperCollector,
}
impl<'a, T: ioWrite> CodeWriter<'a, T> {
    pub fn new(writer: T, option: Rc<CodeGenerateOption>, sfc_info: &'a SFCInfo<'a>) -> Self {
        Self {
            writer: WriteAdaptor::new(writer),
            option,
            sfc_info,
            indent_level: 0,
            closing_brackets: 0,
            cache_count: 0,
            in_alterable: false,
            helpers: Default::default(),
        }
    }
}

impl<'a, T: ioWrite> CoreCodeGenerator<BaseConvertInfo<'a>> for CodeWriter<'a, T> {
    type Written = Output;
    fn generate_prologue(&mut self, root: &mut BaseRoot<'a>) -> Output {
        self.generate_preamble(&mut root.top_scope)?;
        self.generate_function_signature()?;
        self.generate_with_scope()?;
        self.generate_assets(&root.top_scope)?;
        self.write_str("return ")
    }
    fn generate_epilogue(&mut self) -> Output {
        for _ in 0..self.closing_brackets {
            self.deindent()?;
            self.write_str("}")?;
        }
        debug_assert_eq!(self.indent_level, 0);
        Ok(())
    }
    fn generate_text(&mut self, t: BaseText<'a>) -> Output {
        if t.fast_path {
            return self.gen_concate_str(t.texts);
        }
        self.write_helper(RH::CreateText)?;
        self.write_str("(")?;
        self.gen_concate_str(t.texts)?;
        if t.need_patch {
            self.write_str(", ")?;
            self.write_patch(PatchFlag::TEXT)?;
        }
        self.write_str(")")
    }
    fn generate_if(&mut self, i: BaseIf<'a>) -> Output {
        let mut indent = 0;
        for branch in i.branches {
            if branch.condition.is_none() {
                // should use into_inner but it's unstable
                self.generate_ir(*branch.child)?;
                return self.flush_deindent(indent);
            }
            indent += 1;
            let condition = branch.condition.unwrap();
            self.write_str("(")?;
            self.generate_js_expr(condition)?;
            self.write_str(")")?;
            self.indent()?;
            self.write_str("? ")?;
            self.generate_ir(*branch.child)?;
            self.newline()?;
            self.write_str(": ")?;
        }
        if !self.in_alterable {
            // generate default v-else comment
            let s = if self.option.is_dev { "'v-if'" } else { "''" };
            let comment = Js::Call(RH::CreateComment, vec![Js::Src(s), Js::Src("true")]);
            self.generate_js_expr(comment)?;
        } else {
            // generate undefined for alterable_slots
            self.write_str("undefined")?;
        }
        self.flush_deindent(indent)
    }
    fn generate_for(&mut self, f: BaseFor<'a>) -> Output {
        // skip block creation or Fragment in alterable_slots
        if self.in_alterable {
            return self.generate_render_list(f);
        }
        // write open block
        self.gen_open_block(f.is_stable, move |gen| {
            gen.write_helper(RH::CreateElementBlock)?;
            gen.write_str("(")?;
            gen_v_for_args(gen, f)?;
            gen.write_str(")")
        })
    }
    fn generate_vnode(&mut self, v: BaseVNode<'a>) -> Output {
        self.gen_vnode_with_dir(v)
    }
    fn generate_slot_outlet(&mut self, r: BaseRenderSlot<'a>) -> Output {
        self.write_helper(RH::RenderSlot)?;
        self.write_str("(")?;
        gen_render_slot_args(self, r)?;
        self.write_str(")")
    }
    fn generate_v_slot(&mut self, s: BaseVSlot<'a>) -> Output {
        use Slot::*;
        let flag = (Js::str_lit("_"), Flag(s.slot_flag));
        let stable_obj = s
            .stable_slots
            .into_iter()
            .map(|f| (f.name, SlotFn(f.param, f.body)))
            .chain(iter::once(flag));
        // no alterable, output object literal. e.g. {default: ... }
        if s.alterable_slots.is_empty() {
            return self.gen_obj_props(stable_obj, gen_stable_slot_fn);
        }
        self.write_helper(RH::CreateSlots)?;
        self.write_str("(")?;
        self.gen_obj_props(stable_obj, gen_stable_slot_fn)?;
        self.write_str(", ")?;
        // NB: set in_alterable flag to reuse v-for in slot-fn
        self.in_alterable = true;
        self.generate_children(s.alterable_slots)?;
        debug_assert!(self.in_alterable);
        self.in_alterable = false;
        self.write_str(")")
    }
    fn generate_cache(&mut self, c: BaseCache<'a>) -> Self::Written {
        use C::CacheKind as CK;
        match c.kind {
            CK::Once => {
                write!(self.writer, "_cache[{}] || (", self.cache_count)?;
                self.indent()?;
                self.write_helper(RH::SetBlockTracking)?;
                self.write_str("(-1),")?;
                self.newline()?;
                write!(self.writer, "_cache[{}] = ", self.cache_count)?;
                self.generate_ir(*c.child)?;
                self.write_str(",")?;
                self.newline()?;
                self.write_helper(RH::SetBlockTracking)?;
                self.write_str("(1),")?;
                self.newline()?;
                write!(self.writer, "_cache[{}]", self.cache_count)?;
                self.deindent()?;
                self.write_str(")")?;
            }
            CK::Memo(expr) => {
                self.write_helper(RH::WithMemo)?;
                self.write_str("(")?;
                self.generate_js_expr(expr)?;
                self.write_str(", () => ")?;
                self.generate_ir(*c.child)?;
                write!(self.writer, ", _cache, {})", self.cache_count)?;
            }
            CK::MemoInVFor { expr, v_for_key } => {
                self.write_str("const _memo=(")?;
                self.generate_js_expr(expr)?;
                self.write_str(")")?;
                self.newline()?;
                self.write_str("if (_cached")?;
                if let Some(key) = v_for_key {
                    self.write_str(" && _cache.key === ")?;
                    self.generate_js_expr(key)?;
                }
                self.write_str(" && ")?;
                self.write_helper(RH::IsMemoSame)?;
                self.write_str("(_cached, _memo)) return _cached")?;
                self.newline()?;
                self.write_str("const _item = ")?;
                self.generate_ir(*c.child)?;
                self.newline()?;
                self.write_str("_item.memo = _memo")?;
                self.newline()?;
                self.write_str("return _item")?;
            }
        }
        self.cache_count += 1;
        Ok(())
    }
    fn generate_js_expr(&mut self, expr: Js<'a>) -> Output {
        match expr {
            Js::Src(s) | Js::Param(s) => self.write_str(s),
            Js::Num(n) => write!(self.writer, "{}", n),
            Js::StrLit(mut l) => l.be_js_str().write_to(&mut self.writer),
            Js::Simple(e, _) => e.write_to(&mut self.writer),
            Js::Symbol(s) => self.write_helper(s),
            Js::Props(p) => self.gen_obj_props(p, |gen, v| gen.generate_js_expr(v)),
            Js::Compound(v) => {
                for e in v {
                    self.generate_js_expr(e)?;
                }
                Ok(())
            }
            Js::Array(a) => {
                self.write_str("[")?;
                self.gen_list(a)?;
                self.write_str("]")
            }
            Js::Call(c, args) => {
                self.write_helper(c)?;
                self.write_str("(")?;
                self.gen_list(args)?;
                self.write_str(")")
            }
            Js::FuncSimple { src, cache, .. } => {
                let ty = get_handler_type(src);
                gen_handler(self, ty, cache, |gen| src.write_to(&mut gen.writer))
            }
            Js::FuncCompound {
                body, ty, cache, ..
            } => gen_handler(self, ty, cache, |gen| {
                for e in body {
                    gen.generate_js_expr(e)?;
                }
                Ok(())
            }),
        }
    }
    fn generate_alterable_slot(&mut self, s: BaseSlotFn<'a>) -> Output {
        debug_assert!(self.in_alterable);
        // switch back to normal mode
        self.in_alterable = false;
        self.write_str("{")?;
        self.indent()?;
        self.write_str("name: ")?;
        self.generate_js_expr(s.name)?;
        self.write_str(",")?;
        self.newline()?;
        self.write_str("fn: ")?;
        gen_slot_fn(self, (s.param, s.body))?;
        self.deindent()?;
        self.write_str("}")?;
        self.in_alterable = true;
        Ok(())
    }
    fn generate_comment(&mut self, c: &'a str) -> Output {
        let comment = Js::str_lit(c);
        let call = Js::Call(RH::CreateComment, vec![comment]);
        self.generate_js_expr(call)
    }
}

impl<'a, T: ioWrite> CodeWriter<'a, T> {
    pub fn generate_root(&mut self, mut root: BaseRoot<'a>) -> Output {
        // get top scope entities
        self.helpers = root.top_scope.helpers.clone();

        self.generate_prologue(&mut root)?;
        if root.body.is_empty() {
            self.write_str("null")?;
        } else {
            let ir = if root.body.len() == 1 {
                root.body.pop().unwrap()
            } else {
                IRNode::VNodeCall(VNodeIR {
                    tag: Js::Symbol(RH::Fragment),
                    children: root.body,
                    ..VNodeIR::default()
                })
            };
            self.generate_ir(ir)?;
        }
        self.generate_epilogue()
    }
    /// for import helpers or hoist that not in function
    fn generate_preamble(&mut self, top: &mut TopScope<'a>) -> Output {
        match &self.option.clone().mode {
            ScriptMode::Module {
                runtime_module_name,
            } => self.gen_module_preamble(top, runtime_module_name),
            ScriptMode::Function {
                runtime_global_name,
                ..
            } => self.gen_function_preamble(top, runtime_global_name),
        }
    }
    fn gen_function_preamble(&mut self, top: &mut TopScope<'a>, global_name: &str) -> Output {
        debug_assert!(top.helpers == self.helpers);
        if !self.helpers.is_empty() {
            if self.option.use_with_scope() {
                self.write_str("const _Vue = ")?;
                self.write_str(global_name)?;
                self.newline()?;
                // helpers are declared inside with block, but hoists
                // are lifted out so we need extract hoist helper here.
                if !top.hoists.is_empty() {
                    let hoist_helpers = self.helpers.hoist_helpers();
                    self.gen_helper_destruct(hoist_helpers, global_name)?;
                }
            } else {
                let helper = self.helpers.clone();
                self.gen_helper_destruct(helper, global_name)?;
            }
        }
        self.gen_hoist(top)?;
        self.newline()?;
        self.write_str("return ")
    }
    fn should_gen_scope_id(&self) -> bool {
        self.sfc_info.scope_id.is_some() && matches!(self.option.mode, ScriptMode::Module { .. })
    }
    fn gen_module_preamble(&mut self, top: &mut TopScope<'a>, module_name: &str) -> Output {
        if self.should_gen_scope_id() {
            self.helpers.collect(RH::PushScopeId);
            self.helpers.collect(RH::PopScopeId);
        }
        if !self.helpers.is_empty() {
            let helpers = self.helpers.clone();
            self.gen_helper_import(helpers, module_name)?;
            self.newline()?;
        }
        self.gen_imports(top)?;
        self.gen_hoist(top)?;
        self.newline()?;
        if self.sfc_info.inline {
            self.write_str("export ")
        } else {
            Ok(())
        }
    }
    fn gen_helper_import(&mut self, helpers: HelperCollector, from: &str) -> Output {
        self.write_str("import {")?;
        self.indent()?;
        self.gen_helper_import_list(helpers, "as")?;
        self.deindent()?;
        self.write_str("} from \"")?;
        self.write_str(from)?;
        self.write_str("\"")?;
        self.newline()
    }
    fn gen_helper_destruct(&mut self, helpers: HelperCollector, from: &str) -> Output {
        self.write_str("const {")?;
        self.indent()?;
        self.gen_helper_import_list(helpers, ":")?;
        self.deindent()?;
        self.write_str("} = ")?;
        self.write_str(from)?;
        self.newline()
    }
    fn gen_helper_import_list(&mut self, helpers: HelperCollector, sep: &str) -> Output {
        for rh in helpers.into_iter() {
            self.write_str(rh.helper_str())?;
            self.write_str(sep)?;
            self.write_str(" _")?;
            self.write_str(rh.helper_str())?;
            self.write_str(", ")?;
        }
        Ok(())
    }
    fn gen_imports(&mut self, top: &mut TopScope<'a>) -> Output {
        if top.imports.is_empty() {
            return Ok(());
        }
        // take imports
        let mut imports = vec![];
        std::mem::swap(&mut imports, &mut top.imports);
        for impt in imports {
            self.write_str("import ")?;
            self.generate_js_expr(impt.exp)?;
            self.write_str(" from ")?;
            self.write_str(impt.path)?;
            self.newline()?;
        }
        Ok(())
    }
    fn gen_hoist(&mut self, top: &mut TopScope<'a>) -> Output {
        if top.hoists.is_empty() {
            return Ok(());
        }
        let gen_scope_id = self.should_gen_scope_id();
        if gen_scope_id {
            // generate inlined withScopeId helper
            self.write_str("const _withScopeId = n => (")?;
            self.write_helper(RH::PushScopeId)?;
            let scope_id = self.sfc_info.scope_id.as_ref().unwrap();
            write!(self.writer, "({}),n=n(),", scope_id)?;
            self.write_helper(RH::PopScopeId)?;
            self.write_str("(),n)")?;
            self.newline()?;
        }
        // take hoists
        let mut hoists = vec![];
        std::mem::swap(&mut hoists, &mut top.hoists);
        for (i, hoist) in hoists.into_iter().enumerate() {
            let scope_id_wrapper = gen_scope_id && matches!(hoist, IRNode::VNodeCall { .. });
            let wrapper = if scope_id_wrapper {
                "_withScopeId(() => "
            } else {
                ""
            };
            write!(self.writer, "const _hoisted_{} = {}", i, wrapper)?;
            self.generate_ir(hoist)?;
            if scope_id_wrapper {
                self.write_str(")")?;
            }
            self.newline()?;
        }
        Ok(())
    }
    /// render() or ssrRender() and their parameters
    fn generate_function_signature(&mut self) -> Output {
        let option = &self.sfc_info;
        let args = if !option.binding_metadata.is_empty() && !option.inline {
            "_ctx, _cache, $props, $setup, $data, $options"
        } else {
            "_ctx, _cache"
        };
        // NB: vue uses arrow func for inline mode.
        // but it makes no diff in Vue runtime implementation?
        self.write_str("function render(")?;
        self.write_str(args)?;
        self.write_str(") {")?;
        self.closing_brackets += 1;
        self.indent()
    }
    /// with (ctx) for not prefixIdentifier
    fn generate_with_scope(&mut self) -> Output {
        let helpers = self.helpers.clone();
        if !self.option.use_with_scope() {
            return Ok(());
        }
        self.write_str("with (_ctx) {")?;
        self.closing_brackets += 1;
        self.indent()?;
        if helpers.is_empty() {
            return Ok(());
        }
        // function mode const declarations should be inside with block
        // so it doesn't incur the `in` check cost for every helper access.
        self.gen_helper_destruct(helpers, "_Vue")
    }
    /// component/directive resolution inside render
    fn generate_assets(&mut self, top: &TopScope<'a>) -> Output {
        if !top.components.is_empty() {
            self.newline()?;
            let components = top.components.iter().cloned();
            gen_assets(self, components, RH::ResolveComponent)?;
        }
        if !top.directives.is_empty() {
            self.newline()?;
            let directives = top.directives.iter().cloned();
            gen_assets(self, directives, RH::ResolveDirective)?;
        }
        Ok(())
    }

    fn gen_concate_str(&mut self, t: SmallVec<[Js<'a>; 1]>) -> Output {
        let mut texts = t.into_iter();
        match texts.next() {
            Some(t) => self.generate_js_expr(t)?,
            None => return Ok(()),
        }
        for t in texts {
            self.write_str(" + ")?;
            self.generate_js_expr(t)?;
        }
        Ok(())
    }

    fn generate_children(&mut self, children: Vec<BaseIR<'a>>) -> Output {
        debug_assert!(!children.is_empty());
        let fast = if let IRNode::TextCall(t) = &children[0] {
            t.fast_path
        } else {
            false
        };
        if fast {
            // generate sole text node without []
            let ir = children.into_iter().next().unwrap();
            return self.generate_ir(ir);
        }
        self.write_str("[")?;
        self.indent()?;
        for child in children {
            self.generate_ir(child)?;
            self.write_str(", ")?;
        }
        self.deindent()?;
        self.write_str("]")
    }
    fn generate_render_list(&mut self, f: BaseFor<'a>) -> Output {
        let has_memo = if let IRNode::CacheNode(cn) = &*f.child {
            debug_assert!(matches!(cn.kind, C::CacheKind::MemoInVFor { .. }));
            true
        } else {
            false
        };
        self.write_helper(RH::RenderList)?;
        self.write_str("(")?;
        self.generate_js_expr(f.source)?;
        self.write_str(", ")?;
        let p = f.parse_result;
        let mut params = vec![Some(p.value), p.key, p.index];
        if has_memo {
            params.push(Some(Js::Src("_cached")));
            self.gen_func_expr(params, *f.child, /*need_return*/ false)?;
            write!(self.writer, ", _cache, {}", self.cache_count - 1)?;
        } else {
            self.gen_func_expr(params, *f.child, /*need_return*/ true)?;
        }
        self.write_str(")")
    }
    // TODO: add newline
    fn gen_func_expr(
        &mut self,
        params: Vec<Option<Js<'a>>>,
        body: BaseIR<'a>,
        need_return: bool,
    ) -> Output {
        const PLACE_HOLDER: &[&str] = &[
            "_", "_1", "_2", "_3", "_4", "_5", "_6", "_7", "_8", "_9", "_0",
        ];
        let last = params
            .iter()
            .rposition(Option::is_some)
            .map(|i| i + 1)
            .unwrap_or(0);
        debug_assert!(
            last < PLACE_HOLDER.len(),
            "Too many params to generate placeholder"
        );
        let normalized_params = params
            .into_iter()
            .take(last)
            .enumerate()
            .map(|(i, o)| o.unwrap_or(Js::Src(PLACE_HOLDER[i])));
        self.write_str("(")?;
        self.gen_list(normalized_params)?;
        self.write_str(") => {")?;
        self.indent()?;
        if need_return {
            self.write_str("return ")?;
        }
        self.generate_ir(body)?;
        self.deindent()?;
        self.write_str("}")
    }
    /// generate a comma separated list
    fn gen_list<I>(&mut self, exprs: I) -> Output
    where
        I: IntoIterator<Item = Js<'a>>,
    {
        let mut exprs = exprs.into_iter();
        if let Some(e) = exprs.next() {
            self.generate_js_expr(e)?;
        } else {
            return Ok(());
        }
        for e in exprs {
            self.write_str(", ")?;
            self.generate_js_expr(e)?;
        }
        Ok(())
    }
    fn gen_obj_props<V, P, K>(&mut self, props: P, cont: K) -> Output
    where
        P: IntoIterator<Item = (Js<'a>, V)>,
        K: Fn(&mut Self, V) -> Output,
    {
        let mut props = props.into_iter().peekable();
        if props.peek().is_none() {
            return self.write_str("{}");
        }
        self.write_str("{")?;
        self.indent_level += 1; // don't call newline
        for (key, val) in props {
            self.newline()?;
            self.gen_obj_key(key)?;
            self.write_str(": ")?;
            cont(self, val)?;
            self.write_str(",")?;
        }
        self.deindent()?;
        self.write_str("}")
    }
    fn gen_obj_key(&mut self, key: Js<'a>) -> Output {
        if let Js::StrLit(mut k) = key {
            if is_simple_identifier(k) {
                k.write_to(&mut self.writer)
            } else {
                k.be_js_str().write_to(&mut self.writer)
            }
        } else {
            self.write_str("[")?;
            self.generate_js_expr(key)?;
            self.write_str("]")
        }
    }
    fn gen_vnode_with_dir(&mut self, mut v: BaseVNode<'a>) -> Output {
        if v.directives.is_empty() {
            return self.gen_vnode_with_block(v);
        }
        let dirs = std::mem::take(&mut v.directives);
        self.write_helper(RH::WithDirectives)?;
        self.write_str("(")?;
        self.gen_vnode_with_block(v)?;
        self.write_str(", ")?;
        let dir_arr = runtime_dirs_to_js_arr(dirs);
        self.generate_js_expr(dir_arr)?;
        self.write_str(")")
    }
    fn gen_vnode_with_block(&mut self, v: BaseVNode<'a>) -> Output {
        if !v.is_block {
            return gen_vnode_real(self, v);
        }
        self.gen_open_block(v.disable_tracking, move |gen| gen_vnode_real(gen, v))
    }
    fn gen_open_block<K>(&mut self, no_track: bool, cont: K) -> Output
    where
        K: FnOnce(&mut Self) -> Output,
    {
        self.write_str("(")?;
        self.write_helper(RH::OpenBlock)?;
        self.write_str("(")?;
        if no_track {
            self.write_str("true")?;
        }
        self.write_str("), ")?;
        cont(self)?;
        self.write_str(")")
    }

    fn newline(&mut self) -> Output {
        self.write_str("\n")?;
        // TODO: use exponential adding + lazy static
        for _ in 0..self.indent_level {
            self.write_str("  ")?;
        }
        Ok(())
    }
    fn indent(&mut self) -> Output {
        self.indent_level += 1;
        self.newline()
    }
    fn deindent(&mut self) -> Output {
        debug_assert!(self.indent_level > 0);
        self.indent_level -= 1;
        self.newline()
    }
    fn flush_deindent(&mut self, mut indent: usize) -> Output {
        debug_assert!(self.indent_level >= indent);
        while indent > 0 {
            self.indent_level -= 1;
            indent -= 1;
        }
        Ok(())
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> Output {
        self.writer.write_str(s)
    }

    #[inline(always)]
    fn write_helper(&mut self, h: RH) -> Output {
        debug_assert!(self.helpers.contains(h));
        self.write_str("_")?;
        self.write_str(h.helper_str())
    }
    #[inline(always)]
    fn write_patch(&mut self, flag: PatchFlag) -> Output {
        if self.option.is_dev {
            write!(self.writer, "{} /*{:?}*/", flag.bits(), flag)
        } else {
            write!(self.writer, "{}", flag.bits())
        }
    }
}

fn gen_handler<'a, T, F>(
    gen: &mut CodeWriter<'a, T>,
    ty: HandlerType,
    cache: bool,
    func: F,
) -> Output
where
    T: ioWrite,
    F: FnOnce(&mut CodeWriter<'a, T>) -> Output,
{
    if cache {
        write!(gen.writer, "_cache[{}] || (", gen.cache_count)?;
    }
    match ty {
        HandlerType::FuncExpr => func(gen)?,
        HandlerType::MemberExpr => {
            if cache {
                gen.write_str("(...args) => ")?;
            }
            func(gen)?;
            if cache {
                gen.write_str("?.(...args)")?;
            }
        }
        HandlerType::InlineStmt => {
            gen.write_str("$event => (")?;
            func(gen)?;
            gen.write_str(")")?;
        }
    }
    if cache {
        gen.write_str(")")?;
        gen.cache_count += 1;
    }
    Ok(())
}

fn gen_vnode_real<'a, T: ioWrite>(gen: &mut CodeWriter<'a, T>, v: BaseVNode<'a>) -> Output {
    let call_helper = get_vnode_call_helper(&v);
    gen.write_helper(call_helper)?;
    gen.write_str("(")?;
    gen_vnode_call_args(gen, v)?;
    gen.write_str(")")
}

// no, repeating myself is good. macro is bad
/// Takes generator and, condition/generation code pairs.
/// It first finds the last index to write.
/// then generate code for each arg, filling null if empty
/// util the last index to write is reached.
macro_rules! gen_vnode_args {
    (
    $gen:ident,
    $(
        $condition: expr, { $($generate: tt)* }
    )*) => {
        // 1. find the last index to write
        let mut i = 0;
        let mut j = 0;
        $(
            j += 1;
            if $condition {
                i = j;
            }
        )*
        // 2. write code
        j = -1;
        $(
            j += 1;
            if $condition {
                // write comma separator
                if j > 0 {
                    $gen.write_str(", ")?;
                }
                $($generate)*
            } else if i > j {
                // fill null, add comma since first condition must be true
                $gen.write_str(", null")?;
            } else {
                return Ok(())
            }
        )*
    }

}
// TODO: unit test this monster
/// Generate variadic vnode call argument list separated by comma.
/// VNode arg is a heterogeneous list we need hard code the generation.
fn gen_vnode_call_args<'a, T: ioWrite>(gen: &mut CodeWriter<'a, T>, v: BaseVNode<'a>) -> Output {
    let VNodeIR {
        tag,
        props,
        children,
        patch_flag,
        dynamic_props,
        ..
    } = v;

    gen_vnode_args!(
        gen,
        true, { gen.generate_js_expr(tag)?; }
        props.is_some(), { gen.generate_js_expr(props.unwrap())?; }
        !children.is_empty(), { gen.generate_children(children)?; }
        patch_flag != PatchFlag::empty(), {
            gen.write_patch(patch_flag)?;
        }
        !dynamic_props.is_empty(), {
            let dps = dynamic_props.into_iter().map(Js::StrLit);
            gen.write_str("[")?;
            gen.gen_list(dps)?;
            gen.write_str("]")?;
        }
    );
    Ok(())
}

fn gen_v_for_args<'a, T: ioWrite>(gen: &mut CodeWriter<'a, T>, f: BaseFor<'a>) -> Output {
    let flag = f.fragment_flag;
    gen_vnode_args!(
        gen,
        true, { gen.write_helper(RH::Fragment)?; }
        false, {  }
        true, { gen.generate_render_list(f)?; }
        true, {
            write!(gen.writer, "{} /*{:?}*/", flag.bits(), flag)?;
        }
    );
    Ok(())
}

fn gen_render_slot_args<'a, T: ioWrite>(
    gen: &mut CodeWriter<'a, T>,
    r: BaseRenderSlot<'a>,
) -> Output {
    let RenderSlotIR {
        slot_obj,
        slot_name,
        slot_props,
        fallbacks,
        no_slotted,
    } = r;
    gen.generate_js_expr(slot_obj)?;
    gen.write_str(", ")?;
    gen.generate_js_expr(slot_name)?;
    if let Some(prop) = slot_props {
        gen.write_str(", ")?;
        gen.generate_js_expr(prop)?;
    } else {
        debug_assert!(fallbacks.is_empty() && !no_slotted);
        return Ok(());
    }
    if !fallbacks.is_empty() {
        gen.write_str(", ")?;
        gen.write_str("() => ")?;
        gen.generate_children(fallbacks)?;
    } else if no_slotted {
        gen.write_str(", ")?;
        gen.write_str("undefined")?;
    }
    if no_slotted {
        gen.write_str(", ")?;
        gen.write_str("true")
    } else {
        Ok(())
    }
}

enum Slot<'a> {
    SlotFn(Option<Js<'a>>, Vec<BaseIR<'a>>),
    Flag(SlotFlag),
}
fn gen_stable_slot_fn<'a, T: ioWrite>(gen: &mut CodeWriter<'a, T>, slot: Slot<'a>) -> Output {
    match slot {
        Slot::SlotFn(param, body) => gen_slot_fn(gen, (param, body)),
        Slot::Flag(flag) => {
            write!(gen.writer, "{} /*{:?}*/", flag as u8, flag)
        }
    }
}
fn gen_slot_fn<'a, T: ioWrite>(
    gen: &mut CodeWriter<'a, T>,
    (param, body): (Option<Js<'a>>, Vec<BaseIR<'a>>),
) -> Output {
    gen.write_helper(RH::WithCtx)?;
    gen.write_str("(")?;
    gen.write_str("(")?;
    if let Some(p) = param {
        gen.generate_js_expr(p)?;
    }
    gen.write_str(") => [")?;
    gen.indent()?;
    let mut body = body.into_iter();
    if let Some(b) = body.next() {
        gen.generate_ir(b)?;
    }
    for b in body {
        gen.write_str(", ")?;
        gen.newline()?;
        gen.generate_ir(b)?;
    }
    gen.deindent()?;
    gen.write_str("]")?;
    gen.write_str(")")
}
fn gen_assets<'a, T: ioWrite>(
    gen: &mut CodeWriter<'a, T>,
    assets: impl Iterator<Item = VStr<'a>>,
    resolver: RH,
) -> Output {
    for asset in assets {
        let hint = if VStr::is_self_suffixed(&asset) {
            ", true"
        } else {
            ""
        };
        gen.write_str("const ")?;
        asset.write_to(&mut gen.writer)?;
        gen.write_str(" = ")?;
        gen.write_helper(resolver)?;
        gen.write_str("(")?;
        let raw = if resolver == RH::ResolveComponent {
            *asset.clone().unbe_component()
        } else {
            *asset.clone().unbe_directive()
        };
        raw.write_to(&mut gen.writer)?;
        gen.write_str(hint)?;
        gen.write_str(")")?;
        gen.newline()?;
    }
    Ok(())
}

fn runtime_dir(dir: RuntimeDir<BaseConvertInfo>) -> Js {
    let arr = vec![Some(dir.name), dir.expr, dir.arg, dir.mods];
    let last = arr
        .iter()
        .rposition(Option::is_some)
        .map(|i| i + 1)
        .unwrap_or(0);
    let arr = arr
        .into_iter()
        .take(last)
        .map(|o| o.unwrap_or(Js::Src("void 0")))
        .collect();
    Js::Array(arr)
}

fn runtime_dirs_to_js_arr(dirs: Vec<RuntimeDir<BaseConvertInfo>>) -> Js {
    let dirs = dirs.into_iter().map(runtime_dir).collect();
    Js::Array(dirs)
}

#[cfg(test)]
mod test {
    use crate::converter::test::{base_convert, handler_convert};
    use super::*;
    use crate::cast;
    use crate::{BindingMetadata, BindingTypes};
    fn gen<'a>(mut ir: BaseRoot<'a>, info: &'a SFCInfo<'a>) -> String {
        ir.top_scope.helpers.ignore_missing();
        let mut writer = CodeWriter::new(vec![], Default::default(), info);
        writer.generate_root(ir).unwrap();
        String::from_utf8(writer.writer.inner).unwrap()
    }
    fn base_gen(s: &str) -> String {
        let ir = base_convert(s);
        let info = SFCInfo::default();
        gen(ir, &info)
    }
    #[test]
    fn test_text() {
        let s = base_gen("hello       world");
        assert!(s.contains(stringify!("hello world")));
        let s = base_gen("hello {{world}}");
        assert!(s.contains(stringify!("hello ")));
        assert!(s.contains("_createTextVNode(_toDisplayString(world))"));
    }
    #[test]
    fn test_text_merge() {
        let info = SFCInfo::default();
        let mut ir = base_convert("hello{{world}}");
        let world = ir.body.pop().unwrap();
        let world = cast!(world, IRNode::TextCall);
        let hello = cast!(&mut ir.body[0], IRNode::TextCall);
        hello.texts.extend(world.texts);
        let s = gen(ir, &info);
        assert!(s.contains("\"hello\" + _toDisplayString(world)"), "{}", s);
    }
    #[test]
    fn test_text_fast_path() {
        let mut ir = base_convert("hello");
        let hello = cast!(&mut ir.body[0], IRNode::TextCall);
        hello.fast_path = true;
        let s = gen(ir, &SFCInfo::default());
        assert!(!s.contains("_createTextVNode"), "{}", s);
    }
    #[test]
    fn test_v_element() {
        let s = base_gen("<p></p>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains("createElementVNode"), "{}", s);
    }
    #[test]
    fn test_self_closing() {
        let s = base_gen("<p/>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains("createElementVNode"), "{}", s);
        let mut ir = base_convert("<p/>");
        let vn = cast!(&mut ir.body[0], IRNode::VNodeCall);
        vn.is_block = true;
        let s = gen(ir, &SFCInfo::default());
        assert!(s.contains("openBlock"), "{}", s);
    }
    #[test]
    fn test_attr() {
        let s = base_gen("<p class='test' id='id'/>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains(r#"class: "test""#), "{}", s);
        assert!(s.contains(r#"id: "id""#), "{}", s);
        let s = base_gen("<button aria-label='close'/>");
        assert!(s.contains(r#""aria-label": "close""#), "{}", s);
    }
    #[test]
    fn test_v_bind_shorthand() {
        let s = base_gen("<p :prop='id'/>");
        assert!(s.contains("prop: id"), "{}", s);
        let s = base_gen("<p :a='a' :b='b' />");
        assert!(s.contains("a: a,"), "{}", s);
        assert!(s.contains("b: b,"), "{}", s);
        assert!(s.contains("PROPS"), "{}", s);
        let s = base_gen("<p :prop />");
        assert!(s.contains(r#"prop: """#), "{}", s);
    }
    #[test]
    fn test_v_bind_dir() {
        let s = base_gen("<p v-bind:prop='id'/>");
        assert!(s.contains("prop: id"), "{}", s);
        let s = base_gen("<p v-bind=prop />");
        // the below is only in the dom build
        // assert!(s.contains("_normalizeProps(_guardReactiveProps(prop))"), "{}", s);
        assert!(s.contains(", prop, null,"), "{}", s);
        assert!(s.contains("FULL_PROPS"), "{}", s);
        let s = base_gen("<p v-bind=prop class=test />");
        assert!(s.contains("_mergeProps(prop"), "{}", s);
        assert!(s.contains(r#"class: "test""#), "{}", s);
        assert!(s.contains("FULL_PROPS"), "{}", s);
    }

    #[test]
    fn test_v_if() {
        let s = base_gen("<p v-if='condition'/>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains("condition"), "{}", s);
        assert!(s.contains("? "), "{}", s);
        assert!(s.contains("createCommentVNode"), "{}", s);
        let mut ir = base_convert("<p v-if='condition'/>");
        let i = cast!(&mut ir.body[0], IRNode::If);
        let vn = cast!(&mut *i.branches[0].child, IRNode::VNodeCall);
        vn.is_block = true;
        let s = gen(ir, &SFCInfo::default());
        assert!(s.contains("openBlock"), "{}", s);
    }
    #[test]
    fn test_v_if_slot() {
        let s = base_gen("<slot v-if='condition'/>");
        assert!(!s.contains("openBlock"), "{}", s);
        assert!(s.contains("? "), "{}", s);
        assert!(s.contains("createCommentVNode"), "{}", s);
    }

    #[test]
    fn test_v_for() {
        let s = base_gen("<p v-for='a in b'/>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains("(a) =>"), "{}", s);
        assert!(s.contains("_createElementBlock"), "{}", s);
        let s = base_gen("<p v-for='(a, b, c) in d'/>");
        assert!(s.contains("\"p\""), "{}", s);
        assert!(s.contains("(a, b, c) =>"), "{}", s);
    }
    #[test]
    fn test_slot_outlet() {
        let s = base_gen("<slot name=test />");
        assert!(s.contains("_renderSlot"), "{}", s);
        assert!(s.contains(r#", "test""#), "{}", s);
        let s = base_gen("<slot :name=test />");
        assert!(s.contains(", test"), "{}", s);
        let s = base_gen("<slot>fallback</slot>");
        assert!(s.contains("() => ["), "{}", s);
        assert!(s.contains(r#""fallback""#), "{}", s);
    }
    #[test]
    fn test_size() {
        let ir_size = std::mem::size_of::<BaseIR<'_>>();
        let vnode_size = std::mem::size_of::<BaseVNode<'_>>();
        let for_size = std::mem::size_of::<BaseFor<'_>>();
        let js_size = std::mem::size_of::<Js<'_>>();
        let set_size = std::mem::size_of::<std::collections::HashSet<&str>>();
        // TODO: too large
        assert_eq!(ir_size, 184);
        assert_eq!(vnode_size, 152);
        assert_eq!(for_size, 176);
        assert_eq!(js_size, 32);
        assert_eq!(set_size, 48);
    }
    #[test]
    fn test_implicit_slot() {
        let s = base_gen("<component is='test'>test</component>");
        assert!(s.contains("_withCtx"), "{}", s);
    }

    #[test]
    fn test_render_func_args() {
        use rustc_hash::FxHashMap;
        let mut map = FxHashMap::default();
        map.insert("test", BindingTypes::Props);
        let option = SFCInfo {
            binding_metadata: BindingMetadata::new(map, false),
            ..Default::default()
        };
        let ir = base_convert("hello world");
        let s = gen(ir, &option);
        assert!(s.contains("$data"), "{}", s);
        let s = base_gen("hello world");
        assert!(!s.contains("$setup"), "{}", s);
    }

    #[test]
    fn test_v_once() {
        let s = base_gen("<p v-once/>");
        assert!(s.contains("_cache[0]"), "{}", s);
        assert!(s.contains("setBlockTracking"), "{}", s);
    }
    #[test]
    fn test_v_memo() {
        let s = base_gen("<p v-memo='[a]'/>");
        let expected =
            r#"_withMemo([a], () => (_openBlock(), _createElementBlock("p")), _cache, 0)"#;
        assert!(s.contains(expected), "{}", s);
    }
    #[test]
    fn test_v_memo_in_for() {
        let s = base_gen("<p v-for='a in b' v-memo='[a]'/>");
        let expected = r#"_renderList(b, (a, _1, _2, _cached)"#;
        assert!(s.contains(expected), "{}", s);
        assert!(s.contains("_item.memo = _memo"));
        assert!(s.contains("return _item"));
        assert!(!s.contains("_withMemo"), "{}", s);
    }

    fn gen_on(s: &str) -> String {
        let ir = handler_convert(s);
        let info = SFCInfo::default();
        gen(ir, &info)
    }

    #[test]
    fn test_v_model() {
        let s = gen_on("<input v-model='a'/>");
        assert!(s.contains("modelValue: a"), "{}", s);
        assert!(s.contains("\"onUpdate:modelValue\""), "{}", s);
        assert!(s.contains("$event => ((a) = $event)"), "{}", s);
    }

    #[test]
    fn test_handler() {
        // inline statement
        let s = gen_on("<p @click='a()'/>");
        assert!(s.contains("$event => (a())"), "{}", s);
        // member expr
        let s = gen_on("<p @click='a'/>");
        assert!(s.contains("onClick: a"), "{}", s);
        // func expr
        let s = gen_on("<p @click='() => a()'/>");
        assert!(s.contains("onClick: () => a()"), "{}", s);
    }
}
