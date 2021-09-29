use swc_common::BytePos;
use swc_ecma_ast as ast;
use swc_ecma_parser::{
    error::Error as PError, lexer::Lexer, JscTarget, Parser, StringInput, Syntax,
};
use swc_ecma_visit::{Node, Visit};

pub type JsResult = Result<Box<ast::Expr>, PError>;
pub fn parse_js_expr(s: &str) -> JsResult {
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        JscTarget::Es2021,
        StringInput::new(s, BytePos(0), BytePos(s.len() as u32)),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser.parse_expr()
}

struct PrefixIdentifier;

impl Visit for PrefixIdentifier {
    fn visit_ident(&mut self, id: &ast::Ident, parent: &dyn Node) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_js() {
        let a = parse_js_expr("a + b").unwrap();
        println!("{:?}", a);
        panic!("dsfd");
    }
}
