use super::converter::{ConvertInfo, IRNode, IRRoot, JsExpr as Js};
use super::util::VStr;
use rustc_hash::FxHashSet;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::fmt::{Result, Write};

pub trait CodeGenerator {
    type IR;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&self, node: Self::IR) -> Self::Output;
}

pub struct CodeGenerateOption {
    pub is_ts: bool,
    pub source_map: bool,
    // filename for source map
    pub filename: String,
    pub decode_entities: EntityDecoder,
}

pub fn generate_root<T: ConvertInfo>(root: IRRoot<T>) {
    for n in root.body {
        generate_node(n)
    }
}

fn generate_node<T: ConvertInfo>(node: IRNode<T>) {
    use IRNode as IR;
    match node {
        IR::TextCall(..) => generate_text(),
        IR::If(..) => generate_if(),
        IR::For(..) => generate_for(),
        IR::VNodeCall(..) => generate_vnode(),
        IR::RenderSlotCall(..) => generate_slot_outlet(),
        IR::VSlotExpression(..) => generate_v_slot(),
        IR::CommentCall(..) => generate_comment(),
        IR::GenericExpression(..) => generate_js_expr(),
    }
}

// TODO: implement code gen
fn generate_text() {}
fn generate_if() {}
fn generate_for() {}
fn generate_vnode() {}
fn generate_slot_outlet() {}
fn generate_v_slot() {}
fn generate_js_expr() {}
fn generate_comment() {}

pub trait CodeGenWrite: Write {
    fn write_hyphenated(&mut self, s: &str) -> Result {
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
