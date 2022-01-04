use crate::meta_var::{Env, is_meta_var, extract_meta_var, MetaVariable};
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
    println!(
        "goal {}",
        goal.utf8_text(source.as_bytes())
            .expect("invalid source pattern encoding")
    );
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
    loop {
        if let Some(MetaVariable::Ellipsis) = extract_var_from_node(&goal_cursor.node(), source) {
            return Some(candidate);
        }
        match_node_exact(&goal_cursor.node(), candidate_cursor.node(), source, env)?;
        if !goal_cursor.goto_next_sibling() {
            // all goal found, return
            return Some(candidate);
        }
        if !candidate_cursor.goto_next_sibling() {
            return None;
        }
    }
}

fn extract_var_from_node<'tree>(goal: &TNode<'tree>, source: &str) -> Option<MetaVariable> {
    let key = goal
        .utf8_text(source.as_bytes())
        .expect("invalid source pattern encoding");
    extract_meta_var(key)
}

pub fn match_node_recursive<'tree>(
    goal: &TNode<'tree>,
    candidate: TNode<'tree>,
    source: &str,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    let is_leaf = goal.child_count() == 0;
    if is_leaf {
        if let Some(extracted) = extract_var_from_node(goal, source) {
            use MetaVariable as MV;
            match extracted {
                MV::Named(name) => {
                    env.insert(name, candidate);
                    return Some(candidate);
                }
                MV::Anonymous => {
                    return Some(candidate);
                }
                MV::Ellipsis => {
                    return Some(candidate);
                }
                MV::NamedEllipsis(_name) => {
                    todo!("backtracking")
                }
            }
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
        loop {
            if let Some(MetaVariable::Ellipsis) = extract_var_from_node(&goal_cursor.node(), source)
            {
                return Some(candidate);
            }
            match_node_exact(&goal_cursor.node(), candidate_cursor.node(), source, env)?;
            if !goal_cursor.goto_next_sibling() {
                // all goal found, return
                return Some(candidate);
            }
            if !candidate_cursor.goto_next_sibling() {
                return None;
            }
        }
    } else {
        let mut cursor = candidate.walk();
        let mut children = candidate.children(&mut cursor);
        children.find_map(|sub_cand| match_node_recursive(goal, sub_cand, source, env))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::js_parser::parse;
    use std::collections::HashMap;

    fn test_match(s1: &str, s2: &str) -> HashMap<String, String> {
        let goal = parse(s1);
        let goal = goal.root_node().child(0).unwrap();
        let cand = parse(s2);
        let mut env = HashMap::new();
        let ret = match_node_recursive(&goal, cand.root_node(), s1, &mut env);
        assert!(
            ret.is_some(),
            "goal: {}, candidate: {}",
            goal.to_sexp(),
            cand.root_node().to_sexp(),
        );
        env.into_iter()
            .map(|(k, v)| (k, v.utf8_text(s2.as_bytes()).unwrap().into()))
            .collect()
    }

    fn test_non_match(s1: &str, s2: &str) {
        let goal = parse(s1);
        let goal = goal.root_node().child(0).unwrap();
        let cand = parse(s2);
        let mut env = HashMap::new();
        let ret = match_node_recursive(&goal, cand.root_node(), s1, &mut env);
        assert!(ret.is_none());
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
        );
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

    #[test]
    fn test_ellipsis() {
        test_match("foo($$$)", "foo(a, b, c)");
        test_match("foo($$$)", "foo()");
    }
}
