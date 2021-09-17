use super::converter::{BaseConvertInfo, BaseRoot, ConvertInfo, IRNode, IRRoot, JsExpr as Js};
use super::util::VStr;
use rustc_hash::FxHashSet;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::fmt;
use std::io;

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
    fn generate_text(&mut self, t: Vec<T::TextType>) -> Self::Written;
    fn generate_if(&mut self, i: C::IfNodeIR<T>) -> Self::Written;
    fn generate_for(&mut self, f: C::ForNodeIR<T>) -> Self::Written;
    fn generate_vnode(&mut self, v: C::VNodeIR<T>) -> Self::Written;
    fn generate_slot_outlet(&mut self, r: C::RenderSlotIR<T>) -> Self::Written;
    fn generate_v_slot(&mut self, s: C::VSlotIR<T>) -> Self::Written;
    fn generate_js_expr(&mut self, e: T::JsExpression) -> Self::Written;
    fn generate_comment(&mut self, c: T::CommentType) -> Self::Written;
}

struct CodeWriter<'a, T: io::Write> {
    writer: T,
    option: CodeGenerateOption,
    p: std::marker::PhantomData<&'a ()>,
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

impl<'a, T: io::Write> CoreCodeGenerator<BaseConvertInfo<'a>> for CodeWriter<'a, T> {
    type Written = io::Result<()>;
    fn generate_text(&mut self, t: Vec<Js<'a>>) -> io::Result<()> {
        let mut texts = t.into_iter();
        match texts.next() {
            Some(t) => self.generate_one_str(t)?,
            None => return Ok(()),
        }
        for t in texts {
            self.writer.write_all(b" + ")?;
            self.generate_one_str(t)?;
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
        todo!()
    }
    fn generate_slot_outlet(&mut self, r: BaseRenderSlot<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_v_slot(&mut self, s: BaseVSlot<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_js_expr(&mut self, e: Js<'a>) -> io::Result<()> {
        todo!()
    }
    fn generate_comment(&mut self, c: &'a str) -> io::Result<()> {
        todo!()
    }
}

impl<'a, T: io::Write> CodeWriter<'a, T> {
    fn generate_root(&mut self, root: BaseRoot<'a>) -> io::Result<()> {
        use IRNode as IR;
        for node in root.body {
            match node {
                IR::TextCall(t) => self.generate_text(t)?,
                IR::If(v_if) => self.generate_if(v_if)?,
                IR::For(v_for) => self.generate_for(v_for)?,
                IR::VNodeCall(vnode) => self.generate_vnode(vnode)?,
                IR::RenderSlotCall(r) => self.generate_slot_outlet(r)?,
                IR::VSlotUse(s) => self.generate_v_slot(s)?,
                IR::CommentCall(c) => self.generate_comment(c)?,
                IR::GenericExpression(e) => self.generate_js_expr(e)?,
                IR::AlterableSlot(..) => {
                    panic!("alterable slot should be compiled");
                }
            };
        }
        Ok(())
    }
    fn generate_one_str(&mut self, e: Js<'a>) -> io::Result<()> {
        match e {
            Js::StrLit(mut s) => s.be_js_str().write_to(&mut self.writer),
            Js::Simple(s, _) => s.write_to(&mut self.writer),
            Js::Call(..) => self.generate_js_expr(e),
            _ => panic!("wrong text call type"),
        }
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

#[cfg(test)]
mod test {
    use super::super::converter::test::base_convert;
    use super::*;
    #[test]
    fn test_text() {
        let mut writer = CodeWriter {
            writer: vec![],
            option: CodeGenerateOption::default(),
            p: std::marker::PhantomData,
        };
        let ir = base_convert("hello world");
        writer.generate_root(ir).unwrap();
        let s = String::from_utf8(writer.writer).unwrap();
        assert_eq!(s, stringify!("hello world"));
    }
}
