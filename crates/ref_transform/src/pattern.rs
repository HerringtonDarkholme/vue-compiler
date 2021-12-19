use std::collections::HashMap;
use super::Node;
use tree_sitter::{Node as TNode, TreeCursor};

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
    pub fn match_node<'tree>(&'tree self, node: &'tree Node) -> Option<TNode<'tree>> {
        match_node(&self.pattern_node, node)
    }
    pub fn gen_replaced(&self, vars: HashMap<String, String>) -> String {
        todo!()
    }
}

fn match_node<'tree>(
    goal: &'tree Node,
    candidate: &'tree Node,
) -> Option<tree_sitter::Node<'tree>> {
    let goal = goal.inner.root_node();
    if goal.child_count() != 1 {
        todo!("multi-children pattern is not supported yet.")
    }
    let goal = goal.child(0).unwrap();
    let candidate = candidate.inner.root_node();
    if candidate.next_sibling().is_some() {
        todo!("multi candidate roots are not supported yet.")
    }
    match_impl(&goal, candidate)
}
fn match_impl<'tree>(goal: &TNode<'tree>, candidate: TNode<'tree>) -> Option<TNode<'tree>> {
    if goal.kind_id() == candidate.kind_id() {
        if goal.child_count() == 0 {
            return Some(candidate);
        }
        let mut goal_cursor = goal.walk();
        let moved = goal_cursor.goto_first_child();
        debug_assert!(moved);
        let mut candidate_cursor = candidate.walk();
        if !candidate_cursor.goto_first_child() {
            return None;
        }
        while match_impl(&goal_cursor.node(), candidate_cursor.node()).is_some() {
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
        children.find_map(|sub_cand| match_impl(goal, sub_cand))
    }
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
        assert!(match_node(&goal.pattern_node, &cand.pattern_node).is_some());
    }
    fn test_non_match(s1: &str, s2: &str) {
        let goal = Pattern::new(s1);
        let cand = Pattern::new(s2);
        assert!(match_node(&goal.pattern_node, &cand.pattern_node).is_none());
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
}
