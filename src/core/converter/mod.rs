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
pub use super::parser::{AstNode, AstRoot, Directive, Element};
use super::util::{find_dir, VStr};
use rustc_hash::FxHashMap;

mod v_bind;
mod v_for;
mod v_if;
mod v_on;
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
    TextCall(T::TextType),
    /// v-if, else-if, else
    If(IfNodeIR<T>),
    /// v-for
    For(ForNodeIR<T>),
    /// plain element or component
    VNodeCall(T::VNodeType),
    /// <slot> slot outlet
    RenderSlotCall(T::RenderSlotType),
    /// v-slot on component or template
    VSlotExpression(T::VSlotType),
    /// generic JS expression
    GenericExpression(T::JsExpression),
}

pub struct IfNodeIR<T: ConvertInfo> {
    branches: Vec<IfBranch<T>>,
    info: T::IfType,
}
struct IfBranch<T: ConvertInfo> {
    condition: Option<T::JsExpression>,
    children: Vec<IRNode<T>>,
    info: T::IfBranchType,
}
pub struct ForNodeIR<T: ConvertInfo> {
    source: T::JsExpression,
    parse_result: ForParseResult<T>,
    children: Vec<IRNode<T>>,
}
// (value, key, index) in source
struct ForParseResult<T: ConvertInfo> {
    value: Option<T::JsExpression>,
    key: Option<T::JsExpression>,
    index: Option<T::JsExpression>,
}
struct VNodeIR {}

pub type Prop<'a> = (JsExpr<'a>, JsExpr<'a>);
pub enum JsExpr<'a> {
    Lit(&'a str),
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
            AstNode::Text(..) => self.convert_text(),
            AstNode::Comment(..) => self.convert_comment(),
            AstNode::Interpolation(..) => self.convert_interpolation(),
            // all element like node needs pre-convert structural dirs
            AstNode::Plain(e) => pre_convert_for(self, e, |e| self.convert_element(e)),
            AstNode::Component(e) => pre_convert_for(self, e, |e| self.convert_component(e)),
            AstNode::Template(e) => pre_convert_for(self, e, |e| self.convert_template(e)),
            // <slot> requires special v-if/v-for handling
            AstNode::SlotOutlet(..) => self.convert_slot_outlet(),
        }
    }

    // emit error
    fn emit_error(&self, error: CompilationError);
    // core template syntax conversion
    fn convert_directive(&self) -> DirectiveConvertResult<T::JsExpression>;
    fn convert_if(&self, nodes: Vec<Element<'a>>, key: usize) -> IRNode<T>;
    fn convert_for(&self, d: Directive<'a>, n: IRNode<T>) -> IRNode<T>;
    fn convert_slot_outlet(&self) -> IRNode<T>;
    fn convert_element(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_component(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_text(&self) -> IRNode<T>;
    fn convert_interpolation(&self) -> IRNode<T>;
    fn convert_template(&self, e: Element<'a>) -> IRNode<T>;
    fn convert_comment(&self) -> IRNode<T>;
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
    type TextType = &'a str;
    type IfType = ();
    type IfBranchType = usize;
    type ForType = ();
    type VNodeType = ();
    type RenderSlotType = ();
    type VSlotType = ();
    type JsExpression = JsExpr<'a>;
}

type CoreDirConvRet<'a> = DirectiveConvertResult<JsExpr<'a>>;
/// Returns the conversion of a directive. Value could be props or object.
// NB: we pass &dyn ErrorHandler to monomorphize the dir converter to pay
// the minimal cost of dynamism only when error occurs. otherwise we will
// incur the overhead of dyn DirectiveConvert in the ConvertOption.
pub type DirConvertFn =
    for<'a> fn(Directive<'a>, &Element<'a>, &dyn ErrorHandler) -> CoreDirConvRet<'a>;
pub type DirectiveConverter = (&'static str, DirConvertFn);

pub struct BaseConverter {}
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
    fn convert_if(&self, nodes: Vec<Element<'a>>, key: usize) -> BaseIR<'a> {
        v_if::convert_if(self, nodes, key)
    }
    fn convert_for(&self, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
        todo!()
    }
    fn convert_slot_outlet(&self) -> BaseIR<'a> {
        todo!()
    }
    fn convert_element(&self, e: Element<'a>) -> BaseIR<'a> {
        todo!()
    }
    fn convert_component(&self, e: Element<'a>) -> BaseIR<'a> {
        todo!()
    }
    fn convert_text(&self) -> BaseIR<'a> {
        todo!()
    }
    fn convert_interpolation(&self) -> BaseIR<'a> {
        todo!()
    }
    fn convert_template(&self, e: Element<'a>) -> BaseIR<'a> {
        todo!()
    }
    fn convert_comment(&self) -> BaseIR<'a> {
        todo!()
    }
}
