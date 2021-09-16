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

use super::converter as C;
trait CoreCodeGenerator<T: ConvertInfo>: CodeGenerator<IR = IRRoot<T>> {
    fn generate_root(&mut self, root: IRRoot<T>) {
        use IRNode as IR;
        for node in root.body {
            match node {
                IR::TextCall(t) => self.generate_text(t),
                IR::If(v_if) => self.generate_if(v_if),
                IR::For(v_for) => self.generate_for(v_for),
                IR::VNodeCall(vnode) => self.generate_vnode(vnode),
                IR::RenderSlotCall(r) => self.generate_slot_outlet(r),
                IR::VSlotUse(s) => self.generate_v_slot(s),
                IR::CommentCall(c) => self.generate_comment(c),
                IR::GenericExpression(e) => self.generate_js_expr(e),
                IR::AlterableSlot(..) => panic!("alterable slots should be generated inline"),
            }
        }
    }
    fn generate_text(&mut self, t: Vec<T::TextType>);
    fn generate_if(&mut self, i: C::IfNodeIR<T>);
    fn generate_for(&mut self, f: C::ForNodeIR<T>);
    fn generate_vnode(&mut self, v: C::VNodeIR<T>);
    fn generate_slot_outlet(&mut self, r: C::RenderSlotIR<T>);
    fn generate_v_slot(&mut self, s: C::VSlotIR<T>);
    fn generate_js_expr(&mut self, e: T::JsExpression);
    fn generate_comment(&mut self, c: T::CommentType);
}

struct CodeWriter<'a, T: io::Write> {
    writer: T,
    option: CodeGenerateOption,
    p: std::marker::PhantomData<&'a ()>,
}
impl<'a, T: io::Write> CodeGenerator for CodeWriter<'a, T> {
    type IR = BaseRoot<'a>;
    type Output = ();
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
    fn generate_text(&mut self, t: Vec<Js<'a>>) {
        todo!()
    }
    fn generate_if(&mut self, i: BaseIf<'a>) {
        todo!()
    }
    fn generate_for(&mut self, f: BaseFor<'a>) {
        todo!()
    }
    fn generate_vnode(&mut self, v: BaseVNode<'a>) {
        todo!()
    }
    fn generate_slot_outlet(&mut self, r: BaseRenderSlot<'a>) {
        todo!()
    }
    fn generate_v_slot(&mut self, s: BaseVSlot<'a>) {
        todo!()
    }
    fn generate_js_expr(&mut self, e: Js<'a>) {
        todo!()
    }
    fn generate_comment(&mut self, c: &'a str) {
        todo!()
    }
}

pub trait CodeGenWrite: fmt::Write {
    fn write_hyphenated(&mut self, s: &str) -> fmt::Result {
        // JS word boundary is `\w`: `[a-zA-Z0-9-]`.
        // https://javascript.info/regexp-boundary
        // str.replace(/\B([A-Z])/g, '-$1').toLowerCase()
        let mut is_boundary = true;
        for c in s.chars() {
            if !is_boundary && c.is_ascii_uppercase() {
                self.write_char('-')?;
                self.write_char(c.to_ascii_lowercase())?;
                is_boundary = false;
            } else {
                self.write_char(c)?;
                is_boundary = !c.is_ascii_alphanumeric() && c != '_';
            }
        }
        Ok(())
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

fn stringify_dynamic_prop_names(prop_names: FxHashSet<VStr>) -> Option<Js> {
    todo!()
}
