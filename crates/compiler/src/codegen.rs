use crate::converter::RenderSlotIR;

use super::converter::{
    BaseConvertInfo, BaseIR, BaseRoot, ConvertInfo, IRNode, IRRoot, JsExpr as Js, RuntimeDir,
    TopScope, VNodeIR,
};
use super::flags::{HelperCollector, PatchFlag, RuntimeHelper as RH, SlotFlag};
use super::transformer::{
    BaseFor, BaseIf, BaseRenderSlot, BaseSlotFn, BaseText, BaseVNode, BaseVSlot,
};
use crate::util::{get_vnode_call_helper, is_simple_identifier};
use smallvec::{smallvec, SmallVec};
use std::{
    borrow::Cow,
    io::{self, Write},
    iter,
};

pub trait CodeGenerator {
    type IR;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&mut self, node: Self::IR) -> Self::Output;
}

#[derive(PartialEq, Eq)]
pub enum ScriptMode {
    Function { prefix_identifier: bool },
    Module,
}

pub struct CodeGenerateOption {
    pub is_dev: bool,
    pub is_ts: bool,
    pub mode: ScriptMode,
    pub source_map: bool,
    pub inline: bool,
    pub has_binding: bool,
    // filename for source map
    pub filename: String,
    pub decode_entities: EntityDecoder,
}
impl CodeGenerateOption {
    fn use_with_scope(&self) -> bool {
        match self.mode {
            ScriptMode::Function { prefix_identifier } => !prefix_identifier,
            ScriptMode::Module => false,
        }
    }
}
impl Default for CodeGenerateOption {
    fn default() -> Self {
        Self {
            is_dev: true,
            is_ts: false,
            mode: ScriptMode::Function {
                prefix_identifier: false,
            },
            source_map: false,
            inline: false,
            filename: String::new(),
            has_binding: false,
            decode_entities: |s, _| DecodedStr::from(s),
        }
    }
}

use super::converter as C;
trait CoreCodeGenerator<T: ConvertInfo>: CodeGenerator<IR = IRRoot<T>> {
    type Written;
    fn generate_ir(&mut self, ir: IRNode<T>) -> Self::Written {
        use IRNode as IR;
        match ir {
            IR::TextCall(t) => self.generate_text(t),
            IR::If(v_if) => self.generate_if(v_if),
            IR::For(v_for) => self.generate_for(v_for),
            IR::VNodeCall(vnode) => self.generate_vnode(vnode),
            IR::RenderSlotCall(r) => self.generate_slot_outlet(r),
            IR::VSlotUse(s) => self.generate_v_slot(s),
            IR::CommentCall(c) => self.generate_comment(c),
            IR::AlterableSlot(a) => self.generate_alterable_slot(a),
        }
    }
    fn generate_prologue(&mut self, t: &IRRoot<T>) -> Self::Written;
    fn generate_epilogue(&mut self) -> Self::Written;
    fn generate_text(&mut self, t: C::TextIR<T>) -> Self::Written;
    fn generate_if(&mut self, i: C::IfNodeIR<T>) -> Self::Written;
    fn generate_for(&mut self, f: C::ForNodeIR<T>) -> Self::Written;
    fn generate_vnode(&mut self, v: C::VNodeIR<T>) -> Self::Written;
    fn generate_slot_outlet(&mut self, r: C::RenderSlotIR<T>) -> Self::Written;
    fn generate_v_slot(&mut self, s: C::VSlotIR<T>) -> Self::Written;
    fn generate_alterable_slot(&mut self, s: C::Slot<T>) -> Self::Written;
    fn generate_js_expr(&mut self, e: T::JsExpression) -> Self::Written;
    fn generate_comment(&mut self, c: T::CommentType) -> Self::Written;
}

pub struct CodeWriter<'a, T: Write> {
    writer: T,
    indent_level: usize,
    closing_brackets: usize,
    top_scope: TopScope<'a>,
    option: CodeGenerateOption,
}
impl<'a, T: Write> CodeGenerator for CodeWriter<'a, T> {
    type IR = BaseRoot<'a>;
    type Output = io::Result<()>;
    fn generate(&mut self, mut root: Self::IR) -> Self::Output {
        // get top scope entities
        std::mem::swap(&mut self.top_scope, &mut root.top_scope);
        self.generate_root(root)
    }
}

impl<'a, T: Write> CoreCodeGenerator<BaseConvertInfo<'a>> for CodeWriter<'a, T> {
    type Written = io::Result<()>;
    fn generate_prologue(&mut self, root: &BaseRoot<'a>) -> io::Result<()> {
        self.generate_preamble()?;
        self.generate_function_signature()?;
        self.generate_with_scope()?;
        self.generate_assets()?;
        self.write_str("return ")
    }
    fn generate_epilogue(&mut self) -> io::Result<()> {
        for _ in 0..self.closing_brackets {
            self.deindent(true)?;
            self.write_str("}")?;
        }
        debug_assert_eq!(self.indent_level, 0);
        Ok(())
    }
    fn generate_text(&mut self, t: BaseText<'a>) -> io::Result<()> {
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
    fn generate_if(&mut self, i: BaseIf<'a>) -> io::Result<()> {
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
        // generate default v-else comment
        let s = if self.option.is_dev { "'v-if'" } else { "''" };
        let comment = Js::Call(RH::CreateComment, vec![Js::Src(s), Js::Src("true")]);
        self.generate_js_expr(comment)?;
        self.flush_deindent(indent)
    }
    fn generate_for(&mut self, f: BaseFor<'a>) -> io::Result<()> {
        // write open block
        self.gen_open_block(f.is_stable, move |gen| {
            gen.write_helper(RH::CreateElementBlock)?;
            gen.write_str("(")?;
            gen_v_for_args(gen, f)?;
            gen.write_str(")")
        })
    }
    fn generate_vnode(&mut self, v: BaseVNode<'a>) -> io::Result<()> {
        self.gen_vnode_with_dir(v)
    }
    fn generate_slot_outlet(&mut self, r: BaseRenderSlot<'a>) -> io::Result<()> {
        self.write_helper(RH::RenderSlot)?;
        self.write_str("(")?;
        gen_render_slot_args(self, r)?;
        self.write_str(")")
    }
    fn generate_v_slot(&mut self, s: BaseVSlot<'a>) -> io::Result<()> {
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
        // TODO: it's not correct to reuse v-for in slot-fn
        self.generate_children(s.alterable_slots)?;
        self.write_str(")")
    }
    fn generate_js_expr(&mut self, expr: Js<'a>) -> io::Result<()> {
        match expr {
            Js::Src(s) => self.write_str(s),
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
        }
    }
    fn generate_alterable_slot(&mut self, s: BaseSlotFn<'a>) -> io::Result<()> {
        self.write_str("{")?;
        self.indent()?;
        self.write_str("name: ")?;
        self.generate_js_expr(s.name)?;
        self.write_str(",")?;
        self.newline()?;
        self.write_str("fn: ")?;
        gen_slot_fn(self, (s.param, s.body))?;
        self.deindent(true)?;
        self.write_str("}")
    }
    fn generate_comment(&mut self, c: &'a str) -> io::Result<()> {
        let comment = Js::str_lit(c);
        let call = Js::Call(RH::CreateComment, vec![comment]);
        self.generate_js_expr(call)
    }
}

// TODO: put runtimeGlobalName in option
const RUNTIME_GLOBAL_NAME: &str = "Vue";
impl<'a, T: Write> CodeWriter<'a, T> {
    fn generate_root(&mut self, mut root: BaseRoot<'a>) -> io::Result<()> {
        self.generate_prologue(&root)?;
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
    fn generate_preamble(&mut self) -> io::Result<()> {
        if self.option.mode == ScriptMode::Module {
            self.gen_module_preamble()
        } else {
            self.gen_function_preamble()
        }
    }
    fn gen_function_preamble(&mut self) -> io::Result<()> {
        if !self.top_scope.helpers.is_empty() {
            if self.option.use_with_scope() {
                self.write_str("const _Vue = ")?;
                self.write_str(RUNTIME_GLOBAL_NAME)?;
                self.newline()?;
                // helpers are declared inside with block, but hoists
                // are lifted out so we need extract hoist helper here.
                if !self.top_scope.hoists.is_empty() {
                    let hoist_helpers = self.top_scope.helpers.hoist_helpers();
                    self.gen_helper_destruct(hoist_helpers, RUNTIME_GLOBAL_NAME)?;
                }
            } else {
                let helper = self.top_scope.helpers.clone();
                self.gen_helper_destruct(helper, RUNTIME_GLOBAL_NAME)?;
            }
        }
        self.gen_hoist()?;
        self.newline()?;
        self.write_str("return ")
    }
    fn gen_module_preamble(&mut self) -> io::Result<()> {
        todo!()
    }
    fn gen_helper_destruct(&mut self, helpers: HelperCollector, from: &str) -> io::Result<()> {
        self.write_str("const {")?;
        self.indent()?;
        self.gen_helper_import_list(helpers)?;
        self.deindent(true)?;
        self.write_str("} = ")?;
        self.write_str(from)?;
        self.newline()
    }
    fn gen_helper_import_list(&mut self, helpers: HelperCollector) -> io::Result<()> {
        for rh in helpers.into_iter() {
            self.write_str(rh.helper_str())?;
            self.write_str(": _")?;
            self.write_str(rh.helper_str())?;
            self.write_str(", ")?;
        }
        Ok(())
    }
    fn gen_hoist(&mut self) -> io::Result<()> {
        if self.top_scope.hoists.is_empty() {
            return Ok(());
        }
        todo!()
    }
    /// render() or ssrRender() and their parameters
    fn generate_function_signature(&mut self) -> io::Result<()> {
        let option = &self.option;
        let args = if option.has_binding && !option.inline {
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
    fn generate_with_scope(&mut self) -> io::Result<()> {
        let helpers = self.top_scope.helpers.clone();
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
    fn generate_assets(&mut self) -> io::Result<()> {
        let top = &self.top_scope;
        // TODO
        Ok(())
    }

    fn gen_concate_str(&mut self, t: SmallVec<[Js<'a>; 1]>) -> io::Result<()> {
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

    fn generate_children(&mut self, children: Vec<BaseIR<'a>>) -> io::Result<()> {
        self.write_str("[")?;
        self.indent()?;
        for child in children {
            self.generate_ir(child)?;
            self.write_str(", ")?;
        }
        self.deindent(true)?;
        self.write_str("]")
    }
    fn generate_render_list(&mut self, f: BaseFor<'a>) -> io::Result<()> {
        self.write_helper(RH::RenderList)?;
        self.write_str("(")?;
        self.generate_js_expr(f.source)?;
        self.write_str(", ")?;
        let p = f.parse_result;
        let params = vec![Some(p.value), p.key, p.index];
        self.gen_func_expr(params, *f.child)?;
        self.write_str(")")
    }
    // TODO: add newline
    fn gen_func_expr(&mut self, params: Vec<Option<Js<'a>>>, body: BaseIR<'a>) -> io::Result<()> {
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
        self.write_str("return ")?;
        self.generate_ir(body)?;
        self.deindent(true)?;
        self.write_str("}")
    }
    /// generate a comma separated list
    fn gen_list<I>(&mut self, exprs: I) -> io::Result<()>
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
    fn gen_obj_props<V, P, K>(&mut self, props: P, cont: K) -> io::Result<()>
    where
        P: IntoIterator<Item = (Js<'a>, V)>,
        K: Fn(&mut Self, V) -> io::Result<()>,
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
        self.deindent(true)?;
        self.write_str("}")
    }
    fn gen_obj_key(&mut self, key: Js<'a>) -> io::Result<()> {
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
    fn gen_vnode_with_dir(&mut self, mut v: BaseVNode<'a>) -> io::Result<()> {
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
    fn gen_vnode_with_block(&mut self, v: BaseVNode<'a>) -> io::Result<()> {
        if !v.is_block {
            return gen_vnode_real(self, v);
        }
        self.gen_open_block(v.disable_tracking, move |gen| gen_vnode_real(gen, v))
    }
    fn gen_open_block<K>(&mut self, no_track: bool, cont: K) -> io::Result<()>
    where
        K: FnOnce(&mut Self) -> io::Result<()>,
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

    fn newline(&mut self) -> io::Result<()> {
        self.write_str("\n")?;
        for _ in 0..self.indent_level {
            self.write_str("  ")?;
        }
        Ok(())
    }
    fn indent(&mut self) -> io::Result<()> {
        self.indent_level += 1;
        self.newline()
    }
    fn deindent(&mut self, with_new_line: bool) -> io::Result<()> {
        debug_assert!(self.indent_level > 0);
        self.indent_level -= 1;
        if with_new_line {
            self.newline()
        } else {
            Ok(())
        }
    }
    fn flush_deindent(&mut self, mut indent: usize) -> io::Result<()> {
        while indent > 0 {
            self.deindent(false)?;
            indent -= 1;
        }
        Ok(())
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())
    }
    #[inline(always)]
    fn write_helper(&mut self, h: RH) -> io::Result<()> {
        debug_assert!(self.top_scope.helpers.contains(h));
        self.write_str("_")?;
        self.write_str(h.helper_str())
    }
    #[inline(always)]
    fn write_patch(&mut self, flag: PatchFlag) -> io::Result<()> {
        write!(self.writer, "{} /*{:?}*/", flag.bits(), flag)
    }
}

/// DecodedStr represents text after decoding html entities.
/// SmallVec and Cow are used internally for less allocation.
#[derive(Debug)]
pub struct DecodedStr<'a>(SmallVec<[Cow<'a, str>; 1]>);

impl<'a> From<&'a str> for DecodedStr<'a> {
    fn from(decoded: &'a str) -> Self {
        debug_assert!(!decoded.is_empty());
        Self(smallvec![Cow::Borrowed(decoded)])
    }
}

pub type EntityDecoder = fn(&str, bool) -> DecodedStr<'_>;

fn gen_vnode_real<'a, T: Write>(gen: &mut CodeWriter<'a, T>, v: BaseVNode<'a>) -> io::Result<()> {
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
fn gen_vnode_call_args<'a, T: Write>(
    gen: &mut CodeWriter<'a, T>,
    v: BaseVNode<'a>,
) -> io::Result<()> {
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

fn gen_v_for_args<'a, T: Write>(gen: &mut CodeWriter<'a, T>, f: BaseFor<'a>) -> io::Result<()> {
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

fn gen_render_slot_args<'a, T: Write>(
    gen: &mut CodeWriter<'a, T>,
    r: BaseRenderSlot<'a>,
) -> io::Result<()> {
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
fn gen_stable_slot_fn<'a, T: Write>(gen: &mut CodeWriter<'a, T>, slot: Slot<'a>) -> io::Result<()> {
    match slot {
        Slot::SlotFn(param, body) => gen_slot_fn(gen, (param, body)),
        Slot::Flag(flag) => {
            write!(gen.writer, "{} /*{:?}*/", flag as u8, flag)
        }
    }
}
fn gen_slot_fn<'a, T: Write>(
    gen: &mut CodeWriter<'a, T>,
    (param, body): (Option<Js<'a>>, Vec<BaseIR<'a>>),
) -> io::Result<()> {
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
    gen.deindent(true)?;
    gen.write_str("]")?;
    gen.write_str(")")
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
    use super::super::converter::test::base_convert;
    use super::*;
    use crate::cast;
    fn gen(ir: BaseRoot, option: CodeGenerateOption) -> String {
        let mut writer = CodeWriter {
            writer: vec![],
            indent_level: 0,
            closing_brackets: 0,
            top_scope: TopScope::default(),
            option,
        };
        writer.top_scope.helpers.ignore_missing();
        writer.generate_root(ir).unwrap();
        String::from_utf8(writer.writer).unwrap()
    }
    fn base_gen(s: &str) -> String {
        let ir = base_convert(s);
        let option = CodeGenerateOption::default();
        gen(ir, option)
    }
    #[test]
    fn test_text() {
        let s = base_gen("hello       world");
        assert!(s.contains(stringify!("hello world")));
        // let s = base_gen("hello {{world}}");
        // assert!(s.contains("\"hello\" + world"), "{}", s);
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
        let s = gen(ir, CodeGenerateOption::default());
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
        let s = gen(ir, CodeGenerateOption::default());
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
        let option = CodeGenerateOption {
            has_binding: true,
            ..Default::default()
        };
        let ir = base_convert("hello world");
        let s = gen(ir, option);
        assert!(s.contains("$data"), "{}", s);
        let s = base_gen("hello world");
        assert!(!s.contains("$setup"), "{}", s);
    }
}
