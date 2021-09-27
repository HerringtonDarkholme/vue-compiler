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
use super::flags::{HelperCollector, PatchFlag, RuntimeHelper, SlotFlag, StaticLevel};
pub use super::parser::{AstNode, AstRoot, Directive, Element};
use super::parser::{SourceNode, TextNode};
use super::util::{find_dir, VStr};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::{smallvec, SmallVec};
use std::{marker::PhantomData, ops::Deref, rc::Rc};

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
    type TopType: Default;
    // TextType should be a slice of JsExpressions
    type TextType: AsMut<[Self::JsExpression]>;
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
    TextCall(TextIR<T>),
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

pub struct TextIR<T: ConvertInfo> {
    pub fast_path: bool,  // without createTextCall
    pub need_patch: bool, // PatchFlag::TEXT
    pub texts: T::TextType,
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
    pub key: Option<T::JsExpression>,
}
// TODO: optimize as vec to save memory
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
    pub slot_flag: SlotFlag,
}

pub type Prop<'a> = (JsExpr<'a>, JsExpr<'a>);
#[derive(Clone)]
pub enum JsExpr<'a> {
    /// Source. output to generated code as is.
    Src(&'a str),
    /// representing a number, either id or key
    Num(usize),
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
    pub fn str_lit<V: Into<VStr<'a>>>(v: V) -> Self {
        JsExpr::StrLit(v.into())
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

impl BindingTypes {
    pub fn get_js_prop<'a>(&self, name: VStr<'a>, lvl: StaticLevel) -> JsExpr<'a> {
        use BindingTypes::*;
        let obj_dot = JsExpr::Src(match self {
            Data => "$data.",
            Props => "$props.",
            Options => "$options.",
            _ => "$setup.",
        });
        let prop = JsExpr::Simple(name, lvl);
        JsExpr::Compound(vec![obj_dot, prop])
    }
}

pub struct IRRoot<T: ConvertInfo> {
    pub body: Vec<IRNode<T>>,
    /// entities to define/import in top level scope
    pub top_scope: T::TopType,
}

/// Default implementation  sketch can be used in DOM/SSR.
/// Other platform might invent and use their own IR.
pub trait CoreConverter<'a, T: ConvertInfo> {
    fn convert_core_ir(&self, ast: AstRoot<'a>) -> IRRoot<T> {
        let body = self.convert_children(ast.children);
        IRRoot {
            body,
            top_scope: T::TopType::default(),
        }
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
    fn convert_directive(
        &self,
        dir: &mut Directive<'a>,
        e: &mut Element<'a>,
    ) -> DirectiveConvertResult<T::JsExpression>;
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
pub struct BaseConvertInfo<'a>(PhantomData<&'a ()>);

#[derive(Default)]
pub struct TopScope<'a> {
    /// runtime helpers used in template
    pub helpers: HelperCollector,
    /// components that requires resolveComponent call
    pub components: FxHashSet<VStr<'a>>,
    /// directives that requires resolveDirecitve call
    pub directives: FxHashSet<VStr<'a>>,
    /// hoisted vnode/text/js object
    pub hoists: Vec<BaseIR<'a>>,
    /// counters for cached instance, increment per v-once/memo
    pub cached: usize,
    /// counters for temporary variables created in template
    pub temps: usize,
}

impl<'a> ConvertInfo for BaseConvertInfo<'a> {
    type TopType = TopScope<'a>;
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
#[derive(Default)]
pub struct BindingMetadata<'a>(FxHashMap<&'a str, BindingTypes>, bool);
impl<'a> BindingMetadata<'a> {
    pub fn is_setup(&self) -> bool {
        self.1
    }
}
impl<'a> Deref for BindingMetadata<'a> {
    type Target = FxHashMap<&'a str, BindingTypes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct ConvertOption<'a> {
    /// For platform developers. Registers platform specific components written in JS.
    /// e.g. transition, transition-group. Components that require code in Vue runtime.
    pub get_builtin_component: fn(&str) -> Option<RuntimeHelper>,
    pub scope_id: Option<String>,
    /// Indicates this SFC template has used :slotted in its styles
    /// Defaults to `true` for backwards compatibility - SFC tooling should set it
    /// to `false` if no `:slotted` usage is detected in `<style>`
    pub slotted: bool,
    /// Compile the function for inlining inside setup().
    /// This allows the function to directly access setup() local bindings.
    pub inline: bool,
    pub is_dev: bool,
    pub directive_converters: FxHashMap<&'static str, DirConvertFn>,
    /// Optional binding metadata analyzed from script - used to optimize
    /// binding access when `prefixIdentifiers` is enabled.
    pub binding_metadata: Rc<BindingMetadata<'a>>,
    /// current SFC filename for self-referencing
    pub self_name: String,
}

pub struct BaseConverter<'a> {
    pub err_handle: Box<dyn ErrorHandler>,
    pub option: ConvertOption<'a>,
}
pub type BaseRoot<'a> = IRRoot<BaseConvertInfo<'a>>;
pub type BaseIR<'a> = IRNode<BaseConvertInfo<'a>>;
impl<'a> Converter<'a> for BaseConverter<'a> {
    type IR = BaseRoot<'a>;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IR {
        self.convert_core_ir(ast)
    }
}
impl<'a> CoreConverter<'a, BaseConvertInfo<'a>> for BaseConverter<'a> {
    fn emit_error(&self, error: CompilationError) {
        self.err_handle.on_error(error)
    }

    // platform specific methods
    fn get_builtin_component(&self, tag: &str) -> Option<RuntimeHelper> {
        (self.option.get_builtin_component)(tag)
    }

    // core template syntax conversion
    fn convert_directive(
        &self,
        dir: &mut Directive<'a>,
        e: &mut Element<'a>,
    ) -> CoreDirConvRet<'a> {
        if let Some(convert) = self.option.directive_converters.get(dir.name) {
            convert(dir, e, self.err_handle.as_ref())
        } else {
            DirectiveConvertResult::Preserve
        }
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
        let texts = text.text.into_iter().map(JsExpr::StrLit).collect();
        IRNode::TextCall(TextIR {
            fast_path: false,
            need_patch: false,
            texts,
        })
    }
    fn convert_interpolation(&self, interp: SourceNode<'a>) -> BaseIR<'a> {
        let expr = JsExpr::simple(interp.source);
        let call = JsExpr::Call(RuntimeHelper::ToDisplayString, vec![expr]);
        IRNode::TextCall(TextIR {
            fast_path: false,
            need_patch: false,
            texts: smallvec![call],
        })
    }
    fn convert_template(&self, e: Element<'a>) -> BaseIR<'a> {
        convert_element::convert_template(self, e, false)
    }
    fn convert_comment(&self, c: SourceNode<'a>) -> BaseIR<'a> {
        IRNode::CommentCall(c.source)
    }
}

impl<'a> BaseConverter<'a> {
    fn no_slotted(&self) -> bool {
        self.option.scope_id.is_some() && !self.option.slotted
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{cast, error::test::TestErrorHandler, parser::test::base_parse};
    use BaseConverter as BC;
    use JsExpr as Js;

    pub fn assert_str_lit(expr: &Js, s: &str) {
        let v = cast!(expr, Js::StrLit);
        assert_eq!(v.raw, s);
    }
    pub fn assert_simple(expr: &Js, s: &str) {
        let v = cast!(expr, Js::Simple);
        assert_eq!(v.raw, s);
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
        let t = cast!(&body[0], IRNode::TextCall);
        assert_str_lit(&t.texts[0], "hello world");
    }

    #[test]
    fn test_abort() {
        base_convert("hello <p/> {{world}}");
    }

    pub fn base_convert(s: &str) -> BaseRoot {
        let mut convs = FxHashMap::default();
        for (n, f) in [v_bind::V_BIND, ("on", no_op_directive_convert)] {
            convs.insert(n, f);
        }
        let option = ConvertOption {
            get_builtin_component: |_| None,
            scope_id: None,
            slotted: false,
            inline: true,
            is_dev: true,
            directive_converters: convs,
            binding_metadata: Rc::new(BindingMetadata(FxHashMap::default(), false)),
            self_name: "".into(),
        };
        let bc = BC {
            err_handle: Box::new(TestErrorHandler),
            option,
        };
        let ast = base_parse(s);
        bc.convert_ir(ast)
    }
}
