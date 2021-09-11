/*!
IR Converter module takes AST and produces intermediate representation.
All core template syntax conversion happens here. IR is later used for
optimizing transformation and code generation. As we decouple codegen
node from AST, Vue's transformation passes are broken down to two parts.
Convert module roughly corresponds to following transform in vue-next.

# IR Convert
* transformElement
* transformSlotOutlet
* transformTextCall
* vFor
* vIf
* vSlot

# Transform directive
* noopDirectiveTransform
* vModel
* vBind
* vOn
*/

use std::marker::PhantomData;

pub use super::error::{CompilationError, ErrorHandler};
use super::flags::PatchFlag;
pub use super::parser::{AstNode, AstRoot, Directive, Element};
use super::parser::{SourceNode, TextNode};
use super::util::{find_dir, VStr};
use rustc_hash::FxHashMap;

mod build_props;
mod convert_element;
mod convert_slot_outlet;
mod v_bind;
mod v_for;
mod v_if;
mod v_slot;

use v_for::pre_convert_for;
use v_if::{pre_group_v_if, PreGroup};

/// Converts template ast node to intermediate representation.
/// It defines the most generic Converter interface.
/// The IR format can be platform specific.
/// e.g Platfroms other than DOM/SSR can have different IR
pub trait Converter<'a>: Sized {
    type IR;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR;
}

//

pub trait ConvertInfo {
    type TextType;
    type IfType;
    type IfBranchType;
    type ForType;
    type VNodeType;
    type RenderSlotType;
    type VSlotType;
    type CommentType;
    type JsExpression;
}

pub enum VSlotExpr {
    /// stable v-slots declared statically in the template
    StableSlotObject,
    /// v-slots dynamically declared v-slot template with v-if/v-for
    DynamicSlotCall,
}

pub enum IRNode<T: ConvertInfo> {
    /// interpolation or text node
    TextCall(Vec<T::TextType>),
    /// v-if, else-if, else
    If(IfNodeIR<T>),
    /// v-for
    For(ForNodeIR<T>),
    /// component/template/plain element
    VNodeCall(VNodeIR<T>),
    /// <slot> slot outlet
    RenderSlotCall(RenderSlotIR<T>),
    /// v-slot on component or template
    VSlotExpression(T::VSlotType),
    /// comment
    CommentCall(T::CommentType),
    /// generic JS expression
    GenericExpression(T::JsExpression),
}

pub struct IfNodeIR<T: ConvertInfo> {
    branches: Vec<IfBranch<T>>,
    info: T::IfType,
}
struct IfBranch<T: ConvertInfo> {
    condition: Option<T::JsExpression>,
    child: Box<IRNode<T>>,
    info: T::IfBranchType,
}
pub struct ForNodeIR<T: ConvertInfo> {
    source: T::JsExpression,
    parse_result: ForParseResult<T>,
    child: Box<IRNode<T>>,
}
// (value, key, index) in source
struct ForParseResult<T: ConvertInfo> {
    value: T::JsExpression,
    key: Option<T::JsExpression>,
    index: Option<T::JsExpression>,
}
pub struct RenderSlotIR<T: ConvertInfo> {
    slot_name: T::JsExpression,
    slot_props: Option<T::JsExpression>,
    fallbacks: Vec<IRNode<T>>,
    no_slotted: bool,
}

pub struct VNodeIR<T: ConvertInfo> {
    tag: T::JsExpression,
    props: Option<T::JsExpression>,
    children: Vec<IRNode<T>>,
    patch_flag: PatchFlag,
    dynamic_props: Option<T::JsExpression>,
    directives: Vec<DirectiveArgument<T>>,
    is_block: bool,
    disable_tracking: bool,
    is_component: bool,
}

pub struct DirectiveArgument<T: ConvertInfo> {
    dir: T::JsExpression,
    exp: Option<T::JsExpression>,
    arg: Option<T::JsExpression>,
    mods: Option<T::JsExpression>,
}

pub type Prop<'a> = (JsExpr<'a>, JsExpr<'a>);
pub enum JsExpr<'a> {
    /// Source. output to generated code as is.
    Src(&'a str),
    /// String Literal. output after quoted, used by attr/static arg.
    StrLit(VStr<'a>),
    /// will be processed like prefixing
    Simple(VStr<'a>),
    Compound(Vec<JsExpr<'a>>),
    Props(Vec<Prop<'a>>),
    Call(&'static str, Vec<JsExpr<'a>>),
}

pub enum BindingTypes {
    /// returned from data()
    Data,
    /// declared as a prop
    Props,
    /// a let binding (may or may not be a ref)
    SetupLet,
    ///a const binding that can never be a ref.
    ///these bindings don't need `unref()` calls when processed in inlined
    ///template expressions.
    SetupConst,
    /// a const binding that may be a ref.
    SetupMaybeRef,
    /// bindings that are guaranteed to be refs
    SetupRef,
    /// declared by other options, e.g. computed, inject
    Options,
}
pub struct ConvertOption {
    pub directive_converters: Vec<DirectiveConverter>,
    pub binding_metadata: FxHashMap<&'static str, BindingTypes>,
}

pub struct IRRoot<T: ConvertInfo> {
    pub body: Vec<IRNode<T>>,
}

/// Default implementation  sketch can be used in DOM/SSR.
/// Other platform might invent and use their own IR.
pub trait CoreConverter<'a, T>
where
    T: ConvertInfo,
{
    fn convert_core_ir(&self, ast: AstRoot<'a>) -> IRRoot<T> {
        let body = self.convert_children(ast.children);
        IRRoot { body }
    }
    fn convert_children(&self, children: Vec<AstNode<'a>>) -> Vec<IRNode<T>> {
        let mut key = 0;
        // pre group adjacent v-if here to avoid access siblings
        pre_group_v_if(children)
            .map(|pre| match pre {
                PreGroup::VIfGroup(to_convert) => {
                    let len = to_convert.len();
                    let converted = self.convert_if(to_convert, key);
                    key += len;
                    converted
                }
                PreGroup::StandAlone(n) => self.dispatch_ast(n),
            })
            .collect()
    }

    fn dispatch_ast(&self, n: AstNode<'a>) -> IRNode<T> {
        match n {
            AstNode::Text(t) => self.convert_text(t),
            AstNode::Comment(c) => self.convert_comment(c),
            AstNode::Interpolation(i) => self.convert_interpolation(i),
            // all element like node needs pre-convert structural dirs
            AstNode::Element(mut e) => {
                if let Some(dir) = pre_convert_for(&mut e) {
                    self.convert_for(dir, e)
                } else {
                    self.dispatch_element(e)
                }
            }
        }
    }
    fn dispatch_element(&self, e: Element<'a>) -> IRNode<T> {
        use super::parser::ElementType::{SlotOutlet, Template};
        match e.tag_type {
            Template => self.convert_template(e),
            SlotOutlet => self.convert_slot_outlet(e),
            _ => self.convert_element(e),
        }
    }

    // emit error
    fn emit_error(&self, error: CompilationError);
    // core template syntax conversion
    fn convert_directive(&self) -> DirectiveConvertResult<T::JsExpression>;
    fn convert_if(&self, elems: Vec<Element<'a>>, key: usize) -> IRNode<T>;
    fn convert_for(&self, d: Directive<'a>, e: Element<'a>) -> IRNode<T>;
    fn convert_slot_outlet(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_element(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_text(&self, t: TextNode<'a>) -> IRNode<T>;
    fn convert_interpolation(&self, i: SourceNode<'a>) -> IRNode<T>;
    fn convert_template(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_comment(&self, c: SourceNode<'a>) -> IRNode<T>;
}

/// Directive's prop argument passed to VNodeCall after conversion.
/// Use Dropped if the directive is dropped implicitly without codegen.
/// NB: this is not 100% translation from TS. `value` accepts both Props and Object.
// This design decouples v-bind/on from transform_element.
pub enum DirectiveConvertResult<Expr> {
    Converted { value: Expr, need_runtime: bool },
    Dropped,
}

pub fn no_op_directive_convert<'a>(
    _: Directive<'a>,
    _: &Element<'a>,
    _: &dyn ErrorHandler,
) -> DirectiveConvertResult<JsExpr<'a>> {
    DirectiveConvertResult::Dropped
}

// Base Converter for DOM and SSR Fallback

pub struct BaseConvertInfo<'a>(PhantomData<&'a ()>);

impl<'a> ConvertInfo for BaseConvertInfo<'a> {
    type TextType = JsExpr<'a>;
    type IfType = ();
    type IfBranchType = usize;
    type ForType = ();
    type VNodeType = ();
    type RenderSlotType = ();
    type VSlotType = ();
    type CommentType = &'a str;
    type JsExpression = JsExpr<'a>;
}

pub type CoreDirConvRet<'a> = DirectiveConvertResult<JsExpr<'a>>;
/// Returns the conversion of a directive. Value could be props or object.
// NB: we pass &dyn ErrorHandler to monomorphize the dir converter to pay
// the minimal cost of dynamism only when error occurs. otherwise we will
// incur the overhead of dyn DirectiveConvert in the ConvertOption.
pub type DirConvertFn =
    for<'a> fn(Directive<'a>, &Element<'a>, &dyn ErrorHandler) -> CoreDirConvRet<'a>;
pub type DirectiveConverter = (&'static str, DirConvertFn);

pub struct BaseConverter {
    scope_id: Option<String>,
    slotted: bool,
}
type BaseIR<'a> = IRNode<BaseConvertInfo<'a>>;
impl<'a> Converter<'a> for BaseConverter {
    type IR = IRRoot<BaseConvertInfo<'a>>;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR {
        self.convert_core_ir(ast)
    }
}
impl<'a> CoreConverter<'a, BaseConvertInfo<'a>> for BaseConverter {
    // emit error
    fn emit_error(&self, error: CompilationError) {
        todo!()
    }
    // core template syntax conversion
    fn convert_directive(&self) -> CoreDirConvRet<'a> {
        todo!()
    }
    fn convert_if(&self, elems: Vec<Element<'a>>, key: usize) -> BaseIR<'a> {
        v_if::convert_if(self, elems, key)
    }
    fn convert_for(&self, d: Directive<'a>, e: Element<'a>) -> BaseIR<'a> {
        v_for::convert_for(self, d, e)
    }
    fn convert_slot_outlet(&self, e: Element<'a>) -> BaseIR<'a> {
        convert_slot_outlet::convert_slot_outlet(self, e)
    }
    fn convert_element(&self, e: Element<'a>) -> BaseIR<'a> {
        convert_element::convert_element(self, e)
    }
    fn convert_text(&self, text: TextNode<'a>) -> BaseIR<'a> {
        // TODO: reduce allocation by push to existing
        let expr = text.text.into_iter().map(JsExpr::StrLit).collect();
        IRNode::TextCall(expr)
    }
    fn convert_interpolation(&self, interp: SourceNode<'a>) -> BaseIR<'a> {
        let expr = JsExpr::Simple(VStr::raw(interp.source));
        IRNode::TextCall(vec![expr])
    }
    fn convert_template(&self, e: Element<'a>) -> BaseIR<'a> {
        convert_element::convert_template(self, e)
    }
    fn convert_comment(&self, c: SourceNode<'a>) -> BaseIR<'a> {
        IRNode::CommentCall(c.source)
    }
}

impl BaseConverter {
    fn no_slotted(&self) -> bool {
        self.scope_id.is_some() && !self.slotted
    }
}
