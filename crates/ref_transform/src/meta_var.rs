use crate::pattern::PatternKind;
use tree_sitter::Node as TNode;
use crate::Node;
use std::collections::HashMap;

pub type MetaVariableID = String;

pub struct MetaVarEnv<'tree> {
    var_matchers: HashMap<MetaVariableID, MetaVarMatcher>,
    single_matched: HashMap<MetaVariableID, Node<'tree>>,
    multi_matched: HashMap<MetaVariableID, Vec<Node<'tree>>>,
}

impl<'tree> MetaVarEnv<'tree> {
    pub fn insert(&mut self, id: MetaVariableID, ret: Node<'tree>) -> &mut Self {
        self.single_matched.insert(id, ret);
        self
    }

    pub fn insert_multi(&mut self, id: MetaVariableID, ret: Vec<Node<'tree>>) -> &mut Self {
        self.multi_matched.insert(id, ret);
        self
    }

    pub fn get(&self, var: &MetaVariable) -> Option<MatchResult<'tree>> {
        // TODO: optimize this copied/cloned behavior
        match var {
            MetaVariable::Named(n) => self.single_matched.get(n).copied().map(MatchResult::Single),
            MetaVariable::NamedEllipsis(n) => {
                self.multi_matched.get(n).cloned().map(MatchResult::Multi)
            }
            _ => None,
        }
    }
}

impl<'tree> MetaVarEnv<'tree> {
    pub fn match_variable(&self, candidate: TNode<'tree>) -> bool {
        todo!()
    }
    pub fn update_variable(&mut self, candidate: TNode<'tree>) {
        todo!()
    }
}

pub enum MatchResult<'tree> {
    // $A for captured meta var
    Single(Node<'tree>),
    // $$$A for captured ellipsis
    Multi(Vec<Node<'tree>>),
}

pub enum MetaVariable {
    // $A for captured meta var
    Named(MetaVariableID),
    // $_ for non-captured meta var
    Anonymous,
    // $$$ for non-captured ellipsis
    Ellipsis,
    // $$$A for captured ellipsis
    NamedEllipsis(MetaVariableID),
}

pub enum MetaVarMatcher {
    // A regex to filter matched metavar based on its textual content.
    Regex(&'static str),
    // A pattern to filter matched metavar based on its AST tree shape.
    Pattern(PatternKind),
}

pub type Env<'tree> = HashMap<MetaVariableID, Node<'tree>>;

impl MetaVarMatcher {
    pub fn matches(&self, _candidate: &TNode) -> bool {
        todo!()
    }
}

pub fn extract_meta_var(s: &str) -> Option<MetaVariable> {
    use MetaVariable::*;
    if s == "$$$" {
        return Some(Ellipsis);
    }
    if let Some(trimmed) = s.strip_prefix("$$$") {
        if !trimmed.chars().all(is_valid_meta_var_char) {
            return None;
        }
        if trimmed.starts_with('_') {
            return Some(Ellipsis);
        } else {
            return Some(NamedEllipsis(trimmed.to_owned()));
        }
    }
    if !s.starts_with('$') {
        return None;
    }
    let trimmed = &s[1..];
    // $A or $_
    if !trimmed.chars().all(is_valid_meta_var_char) {
        return None;
    }
    if trimmed.starts_with('_') {
        Some(Anonymous)
    } else {
        Some(Named(trimmed.to_owned()))
    }
}

fn is_valid_meta_var_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | '_')
}
