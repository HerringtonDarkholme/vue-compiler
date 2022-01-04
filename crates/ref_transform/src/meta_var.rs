use crate::pattern::PatternKind;
use tree_sitter::Node as TNode;
use std::collections::HashMap;

pub type MetaVariableID = String;

pub struct MetaVarEnv<'tree> {
    var_matchers: HashMap<MetaVariableID, MetaVarMatcher>,
    matched: HashMap<MetaVariableID, TNode<'tree>>,
}

impl<'tree> MetaVarEnv<'tree> {
    pub fn match_variable(&self, candidate: TNode<'tree>) -> bool {
        todo!()
    }
    pub fn update_variable(&mut self, candidate: TNode<'tree>) {
        todo!()
    }
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

pub type Env<'tree> = HashMap<MetaVariableID, TNode<'tree>>;

impl MetaVarMatcher {
    pub fn matches(&self, _candidate: &TNode) -> bool {
        todo!()
    }
}

pub fn is_meta_var(s: &str) -> bool {
    is_single_meta_var(s) || is_ellipsis_meta_var(s)
}

pub fn extract_meta_var(s: &str) -> Option<MetaVariable> {
    use MetaVariable::*;
    println!("{}", s);
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

fn is_single_meta_var(s: &str) -> bool {
    s.starts_with('$') && s[1..].chars().all(is_valid_meta_var_char)
}

fn is_ellipsis_meta_var(s: &str) -> bool {
    // non-captured
    s == "$$$" || s.starts_with("$$$") && s[4..].chars().all(is_valid_meta_var_char)
}
