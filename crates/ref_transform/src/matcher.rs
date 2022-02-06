use crate::meta_var::{Env, extract_meta_var, MetaVariable};
use tree_sitter::{Node as TNode};

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

fn match_leaf_meta_var<'tree>(
    goal: &TNode<'tree>,
    candidate: TNode<'tree>,
    source: &str,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    let extracted = extract_var_from_node(goal, source)?;
    use MetaVariable as MV;
    match extracted {
        MV::Named(name) => {
            env.insert(name, candidate);
            Some(candidate)
        }
        MV::Anonymous => Some(candidate),
        // Ellipsis will be matched in parent level
        MV::Ellipsis => Some(candidate),
        MV::NamedEllipsis(name) => {
            env.insert(name, candidate);
            Some(candidate)
        }
    }
}

fn is_ellipsis<'tree>(node: &TNode<'tree>, source: &str) -> bool {
    matches!(
        extract_var_from_node(node, source),
        Some(MetaVariable::Ellipsis | MetaVariable::NamedEllipsis(_))
    )
}

pub fn match_node_exact<'tree>(
    goal: &TNode<'tree>,
    candidate: TNode<'tree>,
    goal_source: &str,
    cand_source: &str,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    let is_leaf = goal.child_count() == 0;
    if is_leaf {
        if let Some(matched) = match_leaf_meta_var(goal, candidate, goal_source, env) {
            return Some(matched);
        }
    }
    if goal.kind_id() != candidate.kind_id() {
        return None;
    }
    if is_leaf {
        debug_assert!(extract_var_from_node(goal, goal_source).is_none());
        let goal_src = goal
            .utf8_text(goal_source.as_bytes())
            .expect("invalid source pattern encoding");
        let cand_src = candidate
            .utf8_text(cand_source.as_bytes())
            .expect("invalid source pattern encoding");
        return if goal_src == cand_src {
            Some(candidate)
        } else {
            None
        };
    }
    let mut goal_cursor = goal.walk();
    let moved = goal_cursor.goto_first_child();
    debug_assert!(moved);
    let mut candidate_cursor = candidate.walk();
    if !candidate_cursor.goto_first_child() {
        return None;
    }
    loop {
        let curr_node = goal_cursor.node();
        if is_ellipsis(&curr_node, goal_source) {
            // goal has all matched
            if !goal_cursor.goto_next_sibling() {
                // TODO: update env
                return Some(candidate);
            }
            while !goal_cursor.node().is_named() {
                if !goal_cursor.goto_next_sibling() {
                    // TODO: update env
                    return Some(candidate);
                }
            }
            // if next node is a Ellipsis, consume one candidate node
            if is_ellipsis(&goal_cursor.node(), goal_source) {
                if !candidate_cursor.goto_next_sibling() {
                    return None;
                }
                // TODO: update env
                continue;
            }
            loop {
                if match_node_exact(
                    &goal_cursor.node(),
                    candidate_cursor.node(),
                    goal_source,
                    cand_source,
                    env,
                )
                .is_some()
                {
                    // found match non Ellipsis,
                    break;
                }
                if !candidate_cursor.goto_next_sibling() {
                    return None;
                }
            }
        }
        match_node_exact(
            &goal_cursor.node(),
            candidate_cursor.node(),
            goal_source,
            cand_source,
            env,
        )?;
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
    goal_source: &str,
    cand_source: &str,
    env: &mut Env<'tree>,
) -> Option<TNode<'tree>> {
    match_node_exact(goal, candidate, goal_source, cand_source, env).or_else(|| {
        let mut cursor = candidate.walk();
        let mut children = candidate.children(&mut cursor);
        children.find_map(|sub_cand| {
            match_node_recursive(goal, sub_cand, goal_source, cand_source, env)
        })
    })
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
        let ret = match_node_recursive(&goal, cand.root_node(), s1, s2, &mut env);
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
        let ret = match_node_recursive(&goal, cand.root_node(), s1, s2, &mut env);
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
        test_non_match(
            "function foo() { let a = 123; }",
            "function bar() { let a = 123; }",
        );
    }
    #[test]
    fn test_match_inner() {
        test_match(
            "function bar() { let a = 123; }",
            "function foo() { function bar() {let a = 123; }}",
        );
        test_non_match(
            "function foo() { let a = 123; }",
            "function foo() { function bar() {let a = 123; }}",
        );
    }

    #[test]
    fn test_single_ellipsis() {
        test_match("foo($$$)", "foo(a, b, c)");
        test_match("foo($$$)", "foo()");
    }
    #[test]
    fn test_named_ellipsis() {
        test_match("foo($$$A, c)", "foo(a, b, c)");
        test_match("foo($$$A, b, c)", "foo(a, b, c)");
        test_match("foo($$$A, a, b, c)", "foo(a, b, c)");
        test_non_match("foo($$$A, a, b, c)", "foo(b, c)");
    }

    #[test]
    fn test_leading_ellipsis() {
        test_match("foo($$$, c)", "foo(a, b, c)");
        test_match("foo($$$, b, c)", "foo(a, b, c)");
        test_match("foo($$$, a, b, c)", "foo(a, b, c)");
        test_non_match("foo($$$, a, b, c)", "foo(b, c)");
    }
    #[test]
    fn test_trailing_ellipsis() {
        test_match("foo(a, $$$)", "foo(a, b, c)");
        test_match("foo(a, b, $$$)", "foo(a, b, c)");
        // test_match("foo(a, b, c, $$$)", "foo(a, b, c)");
        test_non_match("foo(a, b, c, $$$)", "foo(b, c)");
    }
}
