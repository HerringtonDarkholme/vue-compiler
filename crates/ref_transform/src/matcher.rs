use crate::meta_var::{Env, is_meta_var};
use tree_sitter::Node as TNode;

pub fn match_single_kind<'tree>(
    goal_kind: &str,
    candidate: TNode<'tree>,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    if candidate.kind() == goal_kind {
        // TODO: update env
        // env.insert(meta_var.0.to_owned(), candidate);
        return Some(candidate);
    }
    let mut cursor = candidate.walk();
    let mut children = candidate.children(&mut cursor);
    children.find_map(|sub_cand| match_single_kind(goal_kind, sub_cand, env))
}

pub fn match_node_exact<'tree>(
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
        if is_meta_var(key) {
            env.insert(key.to_owned(), candidate);
            return Some(candidate);
        }
    }
    if goal.kind_id() != candidate.kind_id() {
        return None;
    }
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
    while match_node_exact(&goal_cursor.node(), candidate_cursor.node(), source, env).is_some() {
        if !goal_cursor.goto_next_sibling() {
            // all goal found, return
            return Some(candidate);
        }
        if !candidate_cursor.goto_next_sibling() {
            return None;
        }
    }
    None
}

pub fn match_node_impl<'tree>(
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
        if is_meta_var(key) {
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
        while match_node_exact(&goal_cursor.node(), candidate_cursor.node(), source, env).is_some()
        {
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
        children.find_map(|sub_cand| match_node_impl(goal, sub_cand, source, env))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::js_parser::parse;
    use std::collections::HashMap;

    fn test_match(s1: &str, s2: &str) -> HashMap<String, String> {
        let goal = parse(s1);
        let cand = parse(s2);
        let mut env = HashMap::new();
        let ret = match_node_impl(&goal.root_node(), cand.root_node(), s1, &mut env);
        assert!(ret.is_some());
        env.into_iter()
            .map(|(k, v)| (k, v.utf8_text(s2.as_bytes()).unwrap().into()))
            .collect()
    }

    fn test_non_match(s1: &str, s2: &str) {
        let goal = parse(s1);
        let cand = parse(s2);
        let mut env = HashMap::new();
        let ret = match_node_impl(&goal.root_node(), cand.root_node(), s1, &mut env);
        assert!(ret.is_none());
    }

    #[test]
    fn test_should_exactly_match() {
        test_match(
            "function foo() { let a = 123; }",
            "function foo() { let a = 123; }",
        );
    }
    #[test]
    fn test_extact_match_should_not_match_inner() {
        test_non_match(
            "function foo() { let a = 123; }",
            "function foo() { function bar() {let a = 123; }}",
        );
    }
}
