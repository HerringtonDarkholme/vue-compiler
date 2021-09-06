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

use super::parser::{AstNode, AstRoot, Directive, Element};
use rustc_hash::FxHashMap;

pub trait ConvertInfo {
    type TextType;
    type IfType;
    type ForType;
    type VNodeType;
    type RenderSlotType;
    type VSlotType;
    type GenericJSType;
}

pub enum VSlotExpr {
    /// stable v-slots declared statically in the template
    StableSlotObject,
    /// v-slots dynamically declared v-slot template with v-if/v-for
    DynamicSlotCall,
}

pub enum IRNode<'a, T: ConvertInfo> {
    /// interpolation or text node
    TextCall(&'a str, T::TextType),
    /// v-if, else-if, else
    If(T::IfType),
    /// v-for
    For(T::ForType),
    /// plain element or component
    VNodeCall(T::VNodeType),
    /// <slot> slot outlet
    RenderSlotCall(T::RenderSlotType),
    /// v-slot on component or template
    VSlotExpression(VSlotExpr, T::VSlotType),
    /// generic JS expression
    GenericExpression(T::GenericJSType),
}

type Prop = (String, String);

pub struct DirectiveConvertResult {
    props: Vec<Prop>,
    need_runtime: bool,
}

pub type DirectiveConverter = fn(&Directive, &Element) -> DirectiveConvertResult;

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
    pub directive_converters: Vec<(&'static str, DirectiveConverter)>,
    binding_metadata: FxHashMap<&'static str, BindingTypes>,
}

pub struct IRRoot<'a, T: ConvertInfo> {
    pub body: Vec<IRNode<'a, T>>,
}

/// Converts template ast node to intermediate representation.
/// the IR format can be platform specific.
/// e.g SSR Codegen and DOM Codegen can have different IR
pub trait IRConverter<'a>: Sized {
    type IR;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR;
}

/// Default implementation  sketch can be used in DOM/SSR.
/// Other platform might invent and use their own IR.
pub trait BuiltinConverter<'a, T>
where
    T: ConvertInfo,
    Self: IRConverter<'a, IR = IRRoot<'a, T>>,
{
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR {
        let body = ast
            .children
            .into_iter()
            .map(|n| self.dispatch_ast(n))
            .collect();
        IRRoot { body }
    }
    fn dispatch_ast(&self, n: AstNode<'a>) -> IRNode<'a, T> {
        match n {
            AstNode::Text(..) => self.convert_text(),
            AstNode::Plain(..) => self.convert_element(),
            AstNode::Component(..) => self.convert_element(),
            AstNode::SlotOutlet(..) => self.convert_slot_outlet(),
            AstNode::Comment(..) => self.convert_comment(),
            AstNode::Interpolation(..) => self.convert_interpolation(),
            AstNode::Template(..) => self.convert_template(),
        }
    }
    // core template syntax conversion
    fn convert_directive(&self) -> IRNode<'a, T>;
    fn convert_if(&self) -> IRNode<'a, T>;
    fn convert_for(&self) -> IRNode<'a, T>;
    fn convert_slot_outlet(&self) -> IRNode<'a, T>;
    fn convert_element(&self) -> IRNode<'a, T>;
    fn convert_text(&self) -> IRNode<'a, T>;
    fn convert_interpolation(&self) -> IRNode<'a, T>;
    fn convert_template(&self) -> IRNode<'a, T>;
    fn convert_comment(&self) -> IRNode<'a, T>;
}
