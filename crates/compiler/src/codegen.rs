use super::converter::{
    BaseConvertInfo, BaseRoot, ConvertInfo, IRNode, IRRoot, JsExpr as Js, RuntimeDir, VNodeIR,
};
use super::flags::RuntimeHelper as RH;
use super::util::VStr;
use rustc_hash::FxHashSet;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::fmt;
use std::io;
use std::marker::PhantomData;

pub trait CodeGenerator {
    type IR;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&mut self, node: Self::IR) -> Self::Output;
}

pub struct CodeGenerateOption {
    pub is_ts: bool,
    pub source_map: bool,
    // filename for source map
    pub filename: String,
    pub decode_entities: EntityDecoder,
}
impl Default for CodeGenerateOption {
    fn default() -> Self {
        Self {
            is_ts: false,
            source_map: false,
            filename: String::new(),
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
    fn generate_text(&mut self, t: T::TextType) -> Self::Written;
    fn generate_if(&mut self, i: C::IfNodeIR<T>) -> Self::Written;
    fn generate_for(&mut self, f: C::ForNodeIR<T>) -> Self::Written;
    fn generate_vnode(&mut self, v: C::VNodeIR<T>) -> Self::Written;
    fn generate_slot_outlet(&mut self, r: C::RenderSlotIR<T>) -> Self::Written;
    fn generate_v_slot(&mut self, s: C::VSlotIR<T>) -> Self::Written;
    fn generate_alterable_slot(&mut self, s: C::Slot<T>) -> Self::Written;
    fn generate_js_expr(&mut self, e: T::JsExpression) -> Self::Written;
    fn generate_comment(&mut self, c: T::CommentType) -> Self::Written;
}

struct CodeWriter<'a, T: io::Write> {
    writer: T,
    option: CodeGenerateOption,
    indent_level: usize,
    closing_brackets: usize,
    p: PhantomData<&'a ()>,
}
impl<'a, T: io::Write> CodeGenerator for CodeWriter<'a, T> {
    type IR = BaseRoot<'a>;
    type Output = io::Result<()>;
    fn generate(&mut self, root: Self::IR) -> Self::Output {
        self.generate_root(root)
    }
}

type BaseIf<'a> = C::IfNodeIR<BaseConvertInfo<'a>>;
type BaseFor<'a> = C::ForNodeIR<BaseConvertInfo<'a>>;
type BaseVNode<'a> = C::VNodeIR<BaseConvertInfo<'a>>;
type BaseRenderSlot<'a> = C::RenderSlotIR<BaseConvertInfo<'a>>;
type BaseVSlot<'a> = C::VSlotIR<BaseConvertInfo<'a>>;
type BaseAlterable<'a> = C::Slot<BaseConvertInfo<'a>>;

impl<'a, T: io::Write> CoreCodeGenerator<BaseConvertInfo<'a>> for CodeWriter<'a, T> {
    type Written = io::Result<()>;
    fn generate_prologue(&mut self, root: &BaseRoot<'a>) -> io::Result<()> {
        self.generate_preamble()?;
        self.generate_function_signature()?;
        self.generate_with_block()?;
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
    fn generate_text(&mut self, t: SmallVec<[Js<'a>; 1]>) -> io::Result<()> {
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
    fn generate_if(&mut self, i: BaseIf<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_for(&mut self, f: BaseFor<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_vnode(&mut self, v: BaseVNode<'a>) -> io::Result<()> {
        self.gen_vnode_with_dir(v)
    }
    fn generate_slot_outlet(&mut self, r: BaseRenderSlot<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_v_slot(&mut self, s: BaseVSlot<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_js_expr(&mut self, expr: Js<'a>) -> io::Result<()> {
        match expr {
            Js::Src(s) => self.write_str(s),
            Js::StrLit(mut l) => l.be_js_str().write_to(&mut self.writer),
            Js::Simple(e, _) => e.write_to(&mut self.writer),
            Js::Symbol(s) => self.write_helper(s),
            Js::Props(p) => {
                todo!()
            }
            Js::Compound(v) => {
                for e in v {
                    self.generate_js_expr(e)?;
                }
                Ok(())
            }
            Js::Array(a) => {
                self.write_str("[")?;
                self.gen_comma_separated(a)?;
                self.write_str("]")
            }
            Js::Call(c, args) => {
                self.write_helper(c)?;
                self.write_str("(")?;
                self.gen_comma_separated(args)?;
                self.write_str(")")
            }
        }
    }
    fn generate_alterable_slot(&mut self, s: BaseAlterable<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_comment(&mut self, c: &'a str) -> io::Result<()> {
        todo!()
    }
}

impl<'a, T: io::Write> CodeWriter<'a, T> {
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
        self.write_str("return ")
    }
    /// render() or ssrRender() or IIFE for inline mode
    fn generate_function_signature(&mut self) -> io::Result<()> {
        // TODO: add more params, add more modes
        self.write_str("function render(_ctx, _cache) {")?;
        self.closing_brackets += 1;
        self.indent()
    }
    /// with (ctx) for not prefixIdentifier
    fn generate_with_block(&mut self) -> io::Result<()> {
        // TODO: add helpers
        self.write_str("with (_ctx) {")?;
        self.closing_brackets += 1;
        self.indent()
    }
    /// component/directive resolotuion inside render
    fn generate_assets(&mut self) -> io::Result<()> {
        // TODO
        Ok(())
    }
    fn gen_comma_separated(&mut self, exprs: Vec<Js<'a>>) -> io::Result<()> {
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
            return self.gen_vnode_real(v);
        }
        self.write_str("(")?;
        self.write_helper(RH::OpenBlock)?;
        self.write_str("(")?;
        if v.disable_tracking {
            self.write_str("true")?;
        }
        self.write_str("), ")?;
        self.gen_vnode_real(v)?;
        self.write_str(")")
    }
    fn gen_vnode_real(&mut self, v: BaseVNode<'a>) -> io::Result<()> {
        todo!()
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
        self.indent_level -= 1;
        if with_new_line {
            self.newline()
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())
    }
    #[inline(always)]
    fn write_helper(&mut self, h: RH) -> io::Result<()> {
        self.write_str("_")?;
        self.write_str(h.helper_str())
    }
}

pub trait CodeGenWrite: fmt::Write {}

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

fn stringify_dynamic_prop_names(prop_names: FxHashSet<VStr>) -> Option<Js> {
    todo!()
}

fn runtime_dirs_to_js_arr(_: Vec<RuntimeDir<BaseConvertInfo>>) -> Js {
    todo!()
}

#[cfg(test)]
mod test {
    use super::super::converter::test::base_convert;
    use super::*;
    fn base_gen(s: &str) -> String {
        let mut writer = CodeWriter {
            writer: vec![],
            option: CodeGenerateOption::default(),
            indent_level: 0,
            closing_brackets: 0,
            p: PhantomData,
        };
        let ir = base_convert(s);
        writer.generate_root(ir).unwrap();
        String::from_utf8(writer.writer).unwrap()
    }
    #[test]
    fn test_text() {
        let s = base_gen("hello       world");
        assert!(s.contains(stringify!("hello world")));
        // let s = base_gen("hello {{world}}");
        // assert!(s.contains("\"hello\" + world"), "{}", s);
    }
}
