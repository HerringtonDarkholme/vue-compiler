use std::collections::HashMap;
use super::Node;

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
    pub fn match_node<'cand, 's>(&'s self, node: &'cand Node) -> Option<&'cand Node> {
        match_node(&self.pattern_node, node)
    }
    pub fn gen_replaced(&self, vars: HashMap<String, String>) -> String {
        todo!()
    }
}

fn match_node<'cand, 's>(_goal: &'s Node, _candidate: &'cand Node) -> Option<&'cand Node> {
    todo!()
}

fn extract_meta_vars(_src: &str) -> HashMap<String, MetaVariable> {
    todo!()
}
