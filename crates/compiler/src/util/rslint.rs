use rslint_parser::{
    self as rslint, ast::Expr, tokenize, LossyTreeSink, Parse, Parser, Syntax, TokenSource,
};

// copied from parse_expr
fn parse_expr_lossy(text: &str) -> Parse<Expr> {
    let file_id = 0;
    let (tokens, mut errors) = tokenize(text, file_id);
    let tok_source = TokenSource::new(text, &tokens);
    let mut parser = Parser::new(tok_source, file_id, Syntax::default());
    rslint::syntax::expr::expr(&mut parser);
    let (events, p_diags) = parser.finish();
    errors.extend(p_diags);
    let mut tree_sink = LossyTreeSink::new(text, &tokens);
    rslint::process(&mut tree_sink, events, errors);
    let (green, parse_errors) = tree_sink.finish();
    Parse::new(green, parse_errors)
}

#[cfg(test)]
mod test {
    use super::*;
    use rslint_parser::SyntaxNodeExt;

    #[test]
    fn print_ast() {
        let a = parse_expr_lossy("a + b");
        let b = a.syntax().to::<Expr>();
        println!("{:?}", b);
    }
}
