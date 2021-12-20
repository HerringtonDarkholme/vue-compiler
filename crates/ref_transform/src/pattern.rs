use std::collections::HashMap;
use crate::Node;
use tree_sitter::{Node as TNode};

pub type Env<'tree> = HashMap<String, TNode<'tree>>;

pub struct MetaVariable {
    meta_var_regex: Option<String>,
}

pub struct Pattern {
    meta_variables: HashMap<String, MetaVariable>,
    pattern_node: Node,
}

impl Pattern {
    pub fn new(src: &str) -> Self {
        Self {
            pattern_node: Node::new(src),
            meta_variables: extract_meta_vars(src),
        }
    }
    pub fn match_node<'tree>(&'tree self, node: &'tree Node) -> Option<(TNode<'tree>, Env<'tree>)> {
        match_node(&self.pattern_node, node)
    }
    pub fn gen_replaced(&self, _vars: Env) -> String {
        todo!()
    }
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
    let node = match_impl(&goal, candidate, source, &mut env)?;
    Some((node, env))
}
fn match_impl<'tree>(
    goal: &TNode<'tree>,
    candidate: TNode<'tree>,
    source: &str,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    let is_leaf = goal.child_count() == 0;
    if is_leaf {
        let key = goal
            .utf8_text(source.as_bytes())
            .expect("invalid source pattern encoding");
        if is_wildcard_pattern(key) {
            env.insert(key.to_owned(), candidate);
            return Some(candidate);
        }
    }
    if goal.kind_id() == candidate.kind_id() {
        if is_leaf {
            return Some(candidate);
        }
        let mut goal_cursor = goal.walk();
        let moved = goal_cursor.goto_first_child();
        debug_assert!(moved);
        let mut candidate_cursor = candidate.walk();
        if !candidate_cursor.goto_first_child() {
            return None;
        }
        while match_impl(&goal_cursor.node(), candidate_cursor.node(), source, env).is_some() {
            if !goal_cursor.goto_next_sibling() {
                // all goal found, return
                return Some(candidate);
            }
            if !candidate_cursor.goto_next_sibling() {
                return None;
            }
        }
        None
    } else {
        let mut cursor = candidate.walk();
        let mut children = candidate.children(&mut cursor);
        children.find_map(|sub_cand| match_impl(goal, sub_cand, source, env))
    }
}

fn is_wildcard_pattern(s: &str) -> bool {
    s.starts_with('$') && s[1..].chars().all(|c| matches!(c, 'A'..='Z' | '_'))
}

fn extract_meta_vars(_src: &str) -> HashMap<String, MetaVariable> {
    HashMap::new()
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_match(s1: &str, s2: &str) {
        let goal = Pattern::new(s1);
        let cand = Pattern::new(s2);
        assert!(
            match_node(&goal.pattern_node, &cand.pattern_node).is_some(),
            "goal: {}, candidate: {}",
            goal.pattern_node.inner.root_node().to_sexp(),
            cand.pattern_node.inner.root_node().to_sexp(),
        );
    }
    fn test_non_match(s1: &str, s2: &str) {
        let goal = Pattern::new(s1);
        let cand = Pattern::new(s2);
        assert!(
            match_node(&goal.pattern_node, &cand.pattern_node).is_none(),
            "goal: {}, candidate: {}",
            goal.pattern_node.inner.root_node().to_sexp(),
            cand.pattern_node.inner.root_node().to_sexp(),
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
        let goal = Pattern::new("const a = $VALUE");
        let cand = Pattern::new(cand_str);
        let (_, env) = match_node(&goal.pattern_node, &cand.pattern_node).unwrap();
        assert_eq!(env["$VALUE"].utf8_text(cand_str.as_bytes()).unwrap(), "123");
    }

    #[test]
    fn test_class_assignment() {
        test_match("class $C { $MEMBER = $VAL}", "class A {a = 123}");
        test_non_match("class $C { $MEMBER = $VAL; b = 123; }", "class A {a = 123}");
        // test_match("a = 123", "class A {a = 123}");
        // test_non_match("a = 123", "class B {b = 123}");
    }
}
