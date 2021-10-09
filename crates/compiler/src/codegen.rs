mod code_writer;

use crate::converter::BaseRoot;
use crate::SFCInfo;
use crate::ir::{
    self as C, ConvertInfo, IRNode, IRRoot,
};
use code_writer::CodeWriter;

use smallvec::{smallvec, SmallVec};
use std::marker::PhantomData;
use std::{
    rc::Rc,
    borrow::Cow,
    io::{self, Write as ioWrite},
};

pub trait CodeGenerator {
    type IR<'a>;
    type Info<'a>;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, e.g. String or Binary code or StdOut
    fn generate<'a>(&self, node: Self::IR<'a>, info: Self::Info<'a>) -> Self::Output;
}

#[derive(PartialEq, Eq, Clone)]
pub enum ScriptMode {
    Function {
        /// Transform expressions like {{ foo }} to `_ctx.foo`.
        /// If this option is false, the generated code will be wrapped in a
        /// `with (this) { ... }` block.
        /// - This is force-enabled in module mode, since modules are by default strict
        /// and cannot use `with`
        /// @default mode === 'module'
        prefix_identifier: bool,

        /// Customize the global variable name of `Vue` to get helpers from
        /// in function mode
        /// @default 'Vue'
        runtime_global_name: String,
    },
    Module {
        /// Customize where to import runtime helpers from.
        /// @default 'vue'
        runtime_module_name: String,
    },
}

#[derive(Clone)]
pub struct CodeGenerateOption {
    pub is_dev: bool,
    pub mode: ScriptMode,
    pub source_map: bool,
    pub decode_entities: EntityDecoder,
}
impl CodeGenerateOption {
    fn use_with_scope(&self) -> bool {
        match self.mode {
            ScriptMode::Function {
                prefix_identifier, ..
            } => !prefix_identifier,
            ScriptMode::Module { .. } => false,
        }
    }
}
impl Default for CodeGenerateOption {
    fn default() -> Self {
        Self {
            is_dev: true,
            mode: ScriptMode::Function {
                prefix_identifier: false,
                runtime_global_name: "Vue".into(),
            },
            source_map: false,
            decode_entities: |s, _| DecodedStr::from(s),
        }
    }
}

pub trait CoreCodeGenerator<T: ConvertInfo> {
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
            IR::AlterableSlot(a) => self.generate_alterable_slot(a),
            IR::CacheNode(cache) => self.generate_cache(cache),
            IR::CommentCall(c) => self.generate_comment(c),
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
    fn generate_cache(&mut self, c: C::CacheIR<T>) -> Self::Written;
    fn generate_js_expr(&mut self, e: T::JsExpression) -> Self::Written;
    fn generate_comment(&mut self, c: T::CommentType) -> Self::Written;
}

pub struct CodeGen<T: ioWrite> {
    option: CodeGenerateOption,
    pd: PhantomData<T>,
}
pub struct CodeGenInfo<'a, T: ioWrite> {
    writer: T,
    sfc_info: Rc<SFCInfo<'a>>
}

impl<T: ioWrite> CodeGen<T> {
    pub fn new(option: CodeGenerateOption) -> Self {
        Self {
            option,
            pd: PhantomData,
        }
    }
}


impl<T: ioWrite> CodeGenerator for CodeGen<T> {
    type IR<'a> = BaseRoot<'a>;
    type Info<'a> = CodeGenInfo<'a, T>;
    type Output = io::Result<()>;

    fn generate<'a>(&self, root: BaseRoot<'a>, info: Self::Info<'a>) -> Self::Output {
        let mut imp = CodeWriter::new(info.writer, self.option, info.sfc_info);
        imp.generate_root(root)
            .map_err(|_| imp.writer.get_io_error())
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
