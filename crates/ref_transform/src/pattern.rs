use std::collections::HashMap;
use crate::{Node, Env, MetaVariableID};
use crate::matcher::{match_node_impl, match_single_kind};
use tree_sitter::Node as TNode;

pub enum MetaVarMatcher {
    Regex(&'static str),
    Pattern(PatternKind),
}
impl MetaVarMatcher {
    pub fn matches(&self, _candidate: &TNode) -> bool {
        // todo
        true
    }
}

pub enum PatternKind {
    NodePattern(Node),
    KindPattern(&'static str),
}

pub struct Pattern {
    meta_variables: HashMap<MetaVariableID, MetaVarMatcher>,
    pattern_kind: PatternKind,
}

impl Pattern {
    pub fn new(src: &str) -> Self {
        let node = Node::new(src);
        let pattern_kind = PatternKind::NodePattern(node);
        Self {
            pattern_kind,
            meta_variables: HashMap::new(),
        }
    }
    pub fn of_kind(kind: &'static str) -> Self {
        Self {
            pattern_kind: PatternKind::KindPattern(kind),
            meta_variables: HashMap::new(),
        }
    }
    pub fn match_node<'tree>(&'tree self, node: &'tree Node) -> Option<(TNode<'tree>, Env<'tree>)> {
        match self.pattern_kind {
            PatternKind::NodePattern(ref n) => match_node(n, node),
            PatternKind::KindPattern(k) => match_kind(k, node),
        }
    }

    pub fn with_meta_var<Var: Into<String>>(
        &mut self,
        var_name: Var,
        var_matcher: MetaVarMatcher,
    ) -> &mut Self {
        self.meta_variables.insert(var_name.into(), var_matcher);
        self
    }

    pub fn gen_replaced(&self, _vars: Env) -> String {
        todo!()
    }
}

fn match_kind<'tree>(
    kind: &'static str,
    candidate: &'tree Node,
) -> Option<(TNode<'tree>, Env<'tree>)> {
    let mut env = HashMap::new();
    let candidate = candidate.inner.root_node();
    let node = match_single_kind(kind, candidate, &mut env)?;
    Some((node, env))
}

fn match_node<'tree>(
    goal: &'tree Node,
    candidate: &'tree Node,
) -> Option<(TNode<'tree>, Env<'tree>)> {
    let mut env = HashMap::new();
    let source = &goal.source;
    let goal = goal.inner.root_node();
    if goal.child_count() != 1 {
        todo!("multi-children pattern is not supported yet.")
    }
    let goal = goal.child(0).unwrap();
    let candidate = candidate.inner.root_node();
    if candidate.next_sibling().is_some() {
        todo!("multi candidate roots are not supported yet.")
    }
    let node = match_node_impl(&goal, candidate, source, &mut env)?;
    Some((node, env))
}

#[cfg(test)]
mod test {
    use super::*;

    fn pattern_node(s: &str) -> Node {
        let pattern = Pattern::new(s);
        match pattern.pattern_kind {
            PatternKind::NodePattern(n) => n,
            _ => todo!(),
        }
    }

    fn test_match(s1: &str, s2: &str) {
        let goal = pattern_node(s1);
        let cand = pattern_node(s2);
        assert!(
            match_node(&goal, &cand).is_some(),
            "goal: {}, candidate: {}",
            goal.inner.root_node().to_sexp(),
            cand.inner.root_node().to_sexp(),
        );
    }
    fn test_non_match(s1: &str, s2: &str) {
        let goal = pattern_node(s1);
        let cand = pattern_node(s2);
        assert!(
            match_node(&goal, &cand).is_none(),
            "goal: {}, candidate: {}",
            goal.inner.root_node().to_sexp(),
            cand.inner.root_node().to_sexp(),
        );
    }

    #[test]
    fn test_simple_match() {
        test_match("const a = 123", "const a=123");
        test_non_match("const a = 123", "var a = 123");
    }
    #[test]
    fn test_nested_match() {
        test_match("const a = 123", "function() {const a= 123;}");
        test_match("const a = 123", "class A { constructor() {const a= 123;}}");
        test_match(
            "const a = 123",
            "for (let a of []) while (true) { const a = 123;}",
        )
    }

    #[test]
    fn test_meta_variable() {
        test_match("const a = $VALUE", "const a = 123");
        test_match("const $VARIABLE = $VALUE", "const a = 123");
        test_match("const $VARIABLE = $VALUE", "const a = 123");
    }

    #[test]
    fn test_meta_variable_env() {
        let cand_str = "const a = 123";
        let goal = pattern_node("const a = $VALUE");
        let cand = pattern_node(cand_str);
        let (_, env) = match_node(&goal, &cand).unwrap();
        assert_eq!(env["$VALUE"].utf8_text(cand_str.as_bytes()).unwrap(), "123");
    }

    #[test]
    fn test_match_non_atomic() {
        let cand_str = "const a = 5 + 3";
        let goal = pattern_node("const a = $VALUE");
        let cand = pattern_node(cand_str);
        let (_, env) = match_node(&goal, &cand).unwrap();
        assert_eq!(
            env["$VALUE"].utf8_text(cand_str.as_bytes()).unwrap(),
            "5 + 3"
        );
    }

    #[test]
    fn test_class_assignment() {
        test_match("class $C { $MEMBER = $VAL}", "class A {a = 123}");
        test_non_match("class $C { $MEMBER = $VAL; b = 123; }", "class A {a = 123}");
        // test_match("a = 123", "class A {a = 123}");
        // test_non_match("a = 123", "class B {b = 123}");
    }
}
