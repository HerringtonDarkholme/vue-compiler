use super::parser::{AstNode, AstRoot, Directive};

pub enum IRNode<'a, T: ConvertInfo> {
    Text(&'a str, T::Text),
    Interpolation,
}

type Prop = (String, String);

struct DirectiveConvertResult {
    props: Vec<Prop>,
    need_runtime: bool,
}
type DirectiveConverter = fn(&Directive) -> DirectiveConvertResult;

pub struct ConvertOption {
    directive_converters: Vec<(&'static str, DirectiveConverter)>,
}

pub struct IRRoot<'a, T: ConvertInfo> {
    body: Vec<IRNode<'a, T>>,
}

pub trait ConvertInfo {
    type Text;
}

/// Converts template ast node to intermediate representation.
/// All core template syntax conversion happens here.
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
            AstNode::Text(..) => todo!(),
            AstNode::Plain(..) => self.convert_element(),
            AstNode::Component(..) => self.convert_element(),
            AstNode::SlotOutlet(..) => todo!(),
            AstNode::Template(..) => todo!(),
            AstNode::Comment(..) => todo!(),
            AstNode::Interpolation(..) => todo!(),
        }
    }
    // core template syntax conversion
    fn convert_directive(&self) -> IRNode<'a, T>;
    fn convert_once(&self) -> IRNode<'a, T>;
    fn convert_if(&self) -> IRNode<'a, T>;
    fn convert_memo(&self) -> IRNode<'a, T>;
    fn convert_for(&self) -> IRNode<'a, T>;
    fn convert_slot_outlet(&self) -> IRNode<'a, T>;
    fn convert_element(&self) -> IRNode<'a, T>;
}
