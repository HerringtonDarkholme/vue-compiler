use std::collections::HashMap;
use crate::{Root, Node, meta_var::Env};
use crate::matcher::{match_node_recursive, match_single_kind};
use tree_sitter::Node as TNode;

pub enum PatternKind {
    NodePattern(Root),
    KindPattern(&'static str),
}

pub struct Pattern {
    pattern_kind: PatternKind,
}

impl Pattern {
    pub fn new(src: &str) -> Self {
        let node = Root::new(src);
        let pattern_kind = PatternKind::NodePattern(node);
        Self { pattern_kind }
    }
    pub fn of_kind(kind: &'static str) -> Self {
        Self {
            pattern_kind: PatternKind::KindPattern(kind),
        }
    }
    pub fn match_node<'tree>(&'tree self, node: Node<'tree>) -> Option<(Node<'tree>, Env<'tree>)> {
        match &self.pattern_kind {
            PatternKind::NodePattern(ref n) => {
                let root = n.root();
                match_node(root, node)
            }
            PatternKind::KindPattern(k) => match_kind(k, node),
        }
    }

    pub fn gen_replaced(&self, _vars: Env) -> String {
        todo!()
    }
}

fn match_kind<'tree>(
    kind: &'static str,
    candidate: Node<'tree>,
) -> Option<(Node<'tree>, Env<'tree>)> {
    let mut env = HashMap::new();
    let source = candidate.source;
    let candidate = candidate.inner;
    let inner = match_single_kind(kind, candidate, &mut env)?;
    let node = Node { inner, source };
    Some((node, env))
}

fn match_node<'tree>(
    goal: Node<'tree>,
    candidate: Node<'tree>,
) -> Option<(Node<'tree>, Env<'tree>)> {
    let mut env = HashMap::new();
    let source = &goal.source;
    let cand = &candidate.source;
    let goal = goal.inner;
    if goal.child_count() != 1 {
        todo!("multi-children pattern is not supported yet.")
    }
    let goal = goal.child(0).unwrap();
    let candidate = candidate.inner;
    if candidate.next_sibling().is_some() {
        todo!("multi candidate roots are not supported yet.")
    }
    let node = match_node_recursive(&goal, candidate, source, cand, &mut env)?;
    let node = Node {
        inner: node,
        source: cand,
    };
    Some((node, env))
}

#[cfg(test)]
mod test {
    use super::*;

    fn pattern_node(s: &str) -> Root {
        let pattern = Pattern::new(s);
        match pattern.pattern_kind {
            PatternKind::NodePattern(n) => n,
            _ => todo!(),
        }
    }

    fn test_match(s1: &str, s2: &str) {
        let goal = pattern_node(s1);
        let goal = goal.root();
        let cand = pattern_node(s2);
        let cand = cand.root();
        assert!(
            match_node(goal, cand).is_some(),
            "goal: {}, candidate: {}",
            goal.inner.to_sexp(),
            cand.inner.to_sexp(),
        );
    }
    fn test_non_match(s1: &str, s2: &str) {
        let goal = pattern_node(s1);
        let goal = goal.root();
        let cand = pattern_node(s2);
        let cand = cand.root();
        assert!(
            match_node(goal, cand).is_none(),
            "goal: {}, candidate: {}",
            goal.inner.to_sexp(),
            cand.inner.to_sexp(),
        );
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
        let goal = goal.root();
        let cand = pattern_node(cand_str);
        let cand = cand.root();
        let (_, env) = match_node(goal, cand).unwrap();
        assert_eq!(env["VALUE"].utf8_text(cand_str.as_bytes()).unwrap(), "123");
    }

    #[test]
    fn test_match_non_atomic() {
        let cand_str = "const a = 5 + 3";
        let goal = pattern_node("const a = $VALUE");
        let goal = goal.root();
        let cand = pattern_node(cand_str);
        let cand = cand.root();
        let (_, env) = match_node(goal, cand).unwrap();
        assert_eq!(
            env["VALUE"].utf8_text(cand_str.as_bytes()).unwrap(),
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
