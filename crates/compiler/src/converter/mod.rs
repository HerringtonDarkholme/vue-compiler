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
* vOnce (noop)
* vMemo (noop)

# Transform directive
* noopDirectiveTransform
* vModel
* vBind
* vOn (noop)
*/

pub use super::error::{CompilationError, ErrorHandler};
use super::flags::{self, PatchFlag, RuntimeHelper, StaticLevel};
pub use super::parser::{AstNode, AstRoot, Directive, Element};
use super::parser::{SourceNode, TextNode};
use super::util::{find_dir, VStr};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::{smallvec, SmallVec};

mod build_props;
mod cache_dir;
mod convert_element;
mod convert_slot_outlet;
mod v_bind;
mod v_for;
mod v_if;
mod v_slot;

use cache_dir::{pre_convert_memo, pre_convert_once};
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
    type JsExpression: Default;
    type StrType;
}

pub enum IRNode<T: ConvertInfo> {
    /// interpolation or text node
    TextCall(T::TextType),
    /// v-if, else-if, else
    If(IfNodeIR<T>),
    /// v-for
    For(ForNodeIR<T>),
    /// component/template/plain element
    VNodeCall(VNodeIR<T>),
    /// <slot> slot outlet
    RenderSlotCall(RenderSlotIR<T>),
    /// v-slot used on component or template
    VSlotUse(VSlotIR<T>),
    // internal type for v-slot to reuse v-if/for
    AlterableSlot(Slot<T>),
    /// comment
    CommentCall(T::CommentType),
}

pub struct IfNodeIR<T: ConvertInfo> {
    pub branches: Vec<IfBranch<T>>,
    pub info: T::IfType,
}
pub struct IfBranch<T: ConvertInfo> {
    pub condition: Option<T::JsExpression>,
    pub child: Box<IRNode<T>>,
    pub info: T::IfBranchType,
}
pub struct ForNodeIR<T: ConvertInfo> {
    pub source: T::JsExpression,
    pub parse_result: ForParseResult<T>,
    pub child: Box<IRNode<T>>,
    pub is_stable: bool,
    pub fragment_flag: PatchFlag,
}
// (value, key, index) in source
pub struct ForParseResult<T: ConvertInfo> {
    pub value: T::JsExpression,
    pub key: Option<T::JsExpression>,
    pub index: Option<T::JsExpression>,
}
pub struct RenderSlotIR<T: ConvertInfo> {
    pub slot_obj: T::JsExpression,
    pub slot_name: T::JsExpression,
    pub slot_props: Option<T::JsExpression>,
    pub fallbacks: Vec<IRNode<T>>,
    pub no_slotted: bool,
}
pub struct RuntimeDir<T: ConvertInfo> {
    pub name: T::JsExpression,
    pub expr: Option<T::JsExpression>,
    pub arg: Option<T::JsExpression>,
    pub mods: Option<T::JsExpression>,
}
#[derive(Default)]
pub struct VNodeIR<T: ConvertInfo> {
    pub tag: T::JsExpression,
    pub props: Option<T::JsExpression>,
    pub children: Vec<IRNode<T>>,
    pub patch_flag: PatchFlag,
    pub dynamic_props: FxHashSet<T::StrType>,
    pub directives: Vec<RuntimeDir<T>>,
    pub is_block: bool,
    pub disable_tracking: bool,
    pub is_component: bool,
}
pub struct Slot<T: ConvertInfo> {
    pub name: T::JsExpression,
    pub param: Option<T::JsExpression>,
    pub body: Vec<IRNode<T>>,
}
// note the diffrence between stable and static, dynamic and alterable.
// static = static template name, capturing no identifier
// stable = no if nor for
pub struct VSlotIR<T: ConvertInfo> {
    /// stable v-slots declared statically in the template
    pub stable_slots: Vec<Slot<T>>,
    /// v-slots templates dynamically declared with v-if/v-for
    pub alterable_slots: Vec<IRNode<T>>,
}

pub type Prop<'a> = (JsExpr<'a>, JsExpr<'a>);
pub enum JsExpr<'a> {
    /// Source. output to generated code as is.
    Src(&'a str),
    /// String Literal. output after quoted, used by attr/static arg.
    // TODO: StaticLevel + Simple can mock StrLit?
    StrLit(VStr<'a>),
    /// non-string js expression, will be processed like prefixing
    Simple(VStr<'a>, StaticLevel),
    /// alternative to join string as JsExpr
    Compound(Vec<JsExpr<'a>>),
    Props(Vec<Prop<'a>>),
    /// for calling runtime helper, e.g. resolveComponent()
    Call(RuntimeHelper, Vec<JsExpr<'a>>),
    /// for builtin component called as symbol
    Symbol(RuntimeHelper),
    /// array of JsExpr
    Array(Vec<JsExpr<'a>>),
}

impl<'a> Default for JsExpr<'a> {
    fn default() -> Self {
        Self::Src("")
    }
}

impl<'a> JsExpr<'a> {
    /// a convenient util for creating JsExpr::Simple
    pub fn simple<V: Into<VStr<'a>>>(v: V) -> Self {
        JsExpr::Simple(v.into(), StaticLevel::NotStatic)
    }
    pub fn static_level(&self) -> StaticLevel {
        use JsExpr::*;
        use StaticLevel as S;
        match self {
            Src(_) | StrLit(_) => S::CanStringify,
            Simple(_, level) => *level,
            Compound(v) | Array(v) | Call(_, v) => v
                .iter()
                .map(Self::static_level)
                .min()
                .unwrap_or(S::CanHoist),
            _ => S::NotStatic,
        }
    }
}

#[derive(PartialEq, Eq)]
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
            AstNode::Interpolation(i) => self.convert_interpolation(i),
            AstNode::Comment(c) => self.convert_comment(c),
            // all element like node needs pre-convert structural dirs
            AstNode::Element(e) => self.pre_convert_element(e),
        }
    }
    fn pre_convert_element(&self, mut e: Element<'a>) -> IRNode<T> {
        // order is defined as @vue/compiler-core/src/compile.ts
        let once = pre_convert_once(&mut e);
        let memo = pre_convert_memo(&mut e);
        let vfor = pre_convert_for(&mut e);
        let mut n = self.dispatch_element(e);
        if let Some(d) = vfor {
            n = self.convert_for(d, n);
        }
        if let Some(d) = memo {
            n = self.convert_memo(d, n);
        }
        if let Some(d) = once {
            n = self.convert_once(d, n);
        }
        // reverse order
        n
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
    // platform specific options
    fn get_builtin_component(&self, tag: &str) -> Option<RuntimeHelper>;

    // core template syntax conversion
    fn convert_directive(&self, dir: &mut Directive<'a>)
        -> DirectiveConvertResult<T::JsExpression>;
    fn convert_if(&self, elems: Vec<Element<'a>>, key: usize) -> IRNode<T>;
    fn convert_for(&self, d: Directive<'a>, n: IRNode<T>) -> IRNode<T>;
    fn convert_memo(&self, d: Directive<'a>, n: IRNode<T>) -> IRNode<T>;
    fn convert_once(&self, d: Directive<'a>, n: IRNode<T>) -> IRNode<T>;
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
    Converted {
        value: Expr,
        /// Ok if it needs builtin runtime helper
        /// Err(bool) indicates if it is user defined runtime dir
        runtime: Result<RuntimeHelper, bool>,
    },
    Preserve,
    Dropped,
}

pub fn no_op_directive_convert<'a>(
    _: &mut Directive<'a>,
    _: &Element<'a>,
    _: &dyn ErrorHandler,
) -> DirectiveConvertResult<JsExpr<'a>> {
    DirectiveConvertResult::Dropped
}

// Base Converter for DOM and SSR Fallback
#[derive(Default)]
pub struct BaseConvertInfo<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> ConvertInfo for BaseConvertInfo<'a> {
    type TextType = SmallVec<[JsExpr<'a>; 1]>;
    type IfType = ();
    type IfBranchType = usize;
    type ForType = ();
    type VNodeType = ();
    type RenderSlotType = ();
    type VSlotType = ();
    type CommentType = &'a str;
    type JsExpression = JsExpr<'a>;
    type StrType = VStr<'a>;
}

pub type CoreDirConvRet<'a> = DirectiveConvertResult<JsExpr<'a>>;
/// Returns the conversion of a directive. Value could be props or object.
// NB: we pass &dyn ErrorHandler to monomorphize the dir converter to pay
// the minimal cost of dynamism only when error occurs. otherwise we will
// incur the overhead of dyn DirectiveConvert in the ConvertOption.
pub type DirConvertFn =
    for<'a> fn(&mut Directive<'a>, &Element<'a>, &dyn ErrorHandler) -> CoreDirConvRet<'a>;
pub type DirectiveConverter = (&'static str, DirConvertFn);

/// stores binding variables exposed by data/prop/setup script.
/// also stores if the binding is from setup script.
pub struct BindingMetadata(FxHashMap<&'static str, BindingTypes>, bool);
impl BindingMetadata {
    pub fn is_setup(&self) -> bool {
        self.1
    }
}
impl std::ops::Deref for BindingMetadata {
    type Target = FxHashMap<&'static str, BindingTypes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct BaseConverter {
    pub scope_id: Option<String>,
    /// Indicates this SFC template has used :slotted in its styles
    /// Defaults to `true` for backwards compatibility - SFC tooling should set it
    /// to `false` if no `:slotted` usage is detected in `<style>`
    pub slotted: bool,
    /// Compile the function for inlining inside setup().
    /// This allows the function to directly access setup() local bindings.
    pub inline: bool,
    pub directive_converters: Vec<DirectiveConverter>,
    /// Optional binding metadata analyzed from script - used to optimize
    /// binding access when `prefixIdentifiers` is enabled.
    pub binding_metadata: BindingMetadata,
    /// current SFC filename for self-referencing
    pub self_name: String,
}
pub type BaseRoot<'a> = IRRoot<BaseConvertInfo<'a>>;
pub type BaseIR<'a> = IRNode<BaseConvertInfo<'a>>;
impl<'a> Converter<'a> for BaseConverter {
    type IR = BaseRoot<'a>;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR {
        self.convert_core_ir(ast)
    }
}
impl<'a> CoreConverter<'a, BaseConvertInfo<'a>> for BaseConverter {
    fn emit_error(&self, error: CompilationError) {
        todo!()
    }

    // platform specific methods
    fn get_builtin_component(&self, tag: &str) -> Option<RuntimeHelper> {
        todo!()
    }

    // core template syntax conversion
    fn convert_directive(&self, dr: &mut Directive<'a>) -> CoreDirConvRet<'a> {
        todo!()
    }
    fn convert_if(&self, elems: Vec<Element<'a>>, key: usize) -> BaseIR<'a> {
        v_if::convert_if(self, elems, key)
    }
    fn convert_for(&self, d: Directive<'a>, e: BaseIR<'a>) -> BaseIR<'a> {
        v_for::convert_for(self, d, e)
    }
    // once/memo are noop on SSR/SSR-fallback. They only work in re-render
    fn convert_memo(&self, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
        n
    }
    fn convert_once(&self, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
        n
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
        let expr = JsExpr::simple(interp.source);
        let call = JsExpr::Call(RuntimeHelper::ToDisplayString, vec![expr]);
        IRNode::TextCall(smallvec![call])
    }
    fn convert_template(&self, e: Element<'a>) -> BaseIR<'a> {
        convert_element::convert_template(self, e, false)
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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::parser::test::base_parse;
    use BaseConverter as BC;
    use JsExpr as Js;

    pub fn assert_str_lit(expr: &Js, s: &str) {
        match expr {
            Js::StrLit(v) => assert_eq!(v.raw, s),
            _ => panic!("expr is not string literal"),
        }
    }
    pub fn assert_simple(expr: &Js, s: &str) {
        match expr {
            Js::Simple(v, _) => assert_eq!(v.raw, s),
            _ => panic!("expr is not string literal"),
        }
    }

    #[test]
    fn test_simplest() {
        let body = base_convert("<p/>").body;
        assert_eq!(body.len(), 1);
        if let IRNode::VNodeCall(VNodeIR { tag, .. }) = &body[0] {
            assert_str_lit(tag, "p");
        } else {
            panic!("wrong parsing");
        }
        let body = base_convert("hello world").body;
        if let IRNode::TextCall(texts) = &body[0] {
            assert_str_lit(&texts[0], "hello world");
        } else {
            panic!("wrong parsing");
        }
    }

    pub fn base_convert(s: &str) -> BaseRoot {
        let bc = BC {
            scope_id: None,
            slotted: false,
            inline: true,
            directive_converters: vec![],
            binding_metadata: BindingMetadata(FxHashMap::default(), false),
            self_name: "".into(),
        };
        let ast = base_parse(s);
        bc.convert_ir(ast)
    }
}
