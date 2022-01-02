use crate::pattern::PatternKind;
use tree_sitter::Node as TNode;
use std::collections::HashMap;

pub type MetaVariableID = String;

pub enum MetaVariable {
    // $...A;
    Ellipsis(MetaVariableID),
    // $A
    Single(MetaVariableID),
}

pub enum MetaVarMatcher {
    // A regex to filter matched metavar based on its textual content.
    Regex(&'static str),
    // A pattern to filter matched metavar based on its AST tree shape.
    Pattern(PatternKind),
}

pub type Env<'tree> = HashMap<MetaVariableID, TNode<'tree>>;

impl MetaVarMatcher {
    pub fn matches(&self, _candidate: &TNode) -> bool {
        // todo
        true
    }
}

pub fn is_meta_var(s: &str) -> bool {
    s.starts_with('$') && s[1..].chars().all(|c| matches!(c, 'A'..='Z' | '_'))
}
