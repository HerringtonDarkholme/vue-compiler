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

mod build_props;
mod cache_dir;
mod convert_element;
mod convert_slot_outlet;
mod v_bind;
mod v_for;
mod v_if;
pub mod v_model;
pub mod v_on;
mod v_slot;

use crate::{
    flags::{HelperCollector, RuntimeHelper},
    ir::{ConvertInfo, IRNode, IRRoot, JsExpr, TextIR},
    parser::{SourceNode, TextNode},
    util::{get_core_component, VStr},
    SFCInfo,
};
pub use v_bind::V_BIND;
pub use v_model::V_MODEL;

pub use crate::error::{CompilationError, ErrorHandler, RcErrHandle};
pub use crate::parser::{AstNode, AstRoot, Directive, Element};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::{smallvec, SmallVec};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::Serialize;

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
        // in non reactive build, we can skip cache related dir
        if !self.is_reactive_build() {
            let vfor = pre_convert_for(self, &mut e);
            let mut n = self.dispatch_element(e);
            if let Some(d) = vfor {
                n = self.convert_for(d, n);
            }
            return n;
        }
        // order is defined as @vue/compiler-core/src/compile.ts
        let once = pre_convert_once(&mut e);
        let vfor = pre_convert_for(self, &mut e);
        let memo = pre_convert_memo(&mut e);
        let mut n = self.dispatch_element(e);
        if let Some(d) = memo {
            n = self.convert_memo(d, n);
        }
        if let Some(d) = vfor {
            n = self.convert_for(d, n);
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

    // emit error
    fn emit_error(&self, error: CompilationError);
    // platform specific options
    fn get_builtin_component(&self, tag: &str) -> Option<RuntimeHelper>;
    // is reactive
    fn is_reactive_build(&self) -> bool;
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
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct BaseConvertInfo<'a>(PhantomData<&'a ()>);

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    type IfBranchType = usize;
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

#[derive(Clone)]
pub struct ConvertOption {
    /// For platform developers. Registers platform specific components written in JS.
    /// e.g. transition, transition-group. Components that require code in Vue runtime.
    pub get_builtin_component: fn(&str) -> Option<RuntimeHelper>,
    pub directive_converters: FxHashMap<&'static str, DirConvertFn>,
    pub is_dev: bool,
    pub need_reactivity: bool,
}

impl Default for ConvertOption {
    fn default() -> Self {
        Self {
            get_builtin_component: get_core_component,
            is_dev: true,
            need_reactivity: true,
            directive_converters: FxHashMap::default(),
        }
    }
}

pub struct BaseConverter<'a> {
    pub err_handle: RcErrHandle,
    pub sfc_info: SFCInfo<'a>,
    pub option: ConvertOption,
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
    fn is_reactive_build(&self) -> bool {
        self.option.need_reactivity
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
        if self.is_reactive_build() {
            cache_dir::convert_memo(self, d, n)
        } else {
            n
        }
    }
    fn convert_once(&self, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
        if self.is_reactive_build() {
            cache_dir::convert_once(self, d, n)
        } else {
            n
        }
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
        self.sfc_info.scope_id.is_some() && !self.sfc_info.slotted
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{cast, error::test::TestErrorHandler, ir::VNodeIR, parser::test::base_parse};
    use std::rc::Rc;
    use BaseConverter as BC;
    use JsExpr as Js;

    pub fn assert_str_lit(expr: &Js, s: &str) {
        let v = cast!(expr, Js::StrLit);
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
            directive_converters: convs,
            ..Default::default()
        };
        let bc = BC {
            err_handle: Rc::new(TestErrorHandler),
            sfc_info: Default::default(),
            option,
        };
        let ast = base_parse(s);
        bc.convert_ir(ast)
    }
    pub fn handler_convert(s: &str) -> BaseRoot {
        let convs = vec![
            v_bind::V_BIND,
            v_on::V_ON,
            ("model", v_model::convert_v_model_event),
        ]
        .into_iter()
        .collect();
        let option = ConvertOption {
            directive_converters: convs,
            ..Default::default()
        };
        let bc = BC {
            err_handle: Rc::new(TestErrorHandler),
            sfc_info: Default::default(),
            option,
        };
        let ast = base_parse(s);
        bc.convert_ir(ast)
    }
}
