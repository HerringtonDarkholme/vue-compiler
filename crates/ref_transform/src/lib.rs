use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use tree_sitter::{Node as TNode};
use std::collections::HashMap;

mod js_parser;
mod language;
mod matcher;
mod pattern;
mod rule;

pub use pattern::Pattern;

pub type MetaVariableID = String;
pub type Env<'tree> = HashMap<MetaVariableID, TNode<'tree>>;

pub struct Semgrep {
    root: Node,
}

pub struct Node {
    inner: js_parser::Tree,
    source: Rc<String>,
}

impl Node {
    fn new(src: &str) -> Self {
        Self {
            inner: js_parser::parse(src),
            source: Rc::new(src.into()),
        }
    }
}

// tree traversal API
impl Node {
    pub fn find(&self) -> Node {
        todo!()
    }
    // should we provide parent?
    pub fn parent(&self) -> Node {
        todo!()
    }
    pub fn ancestors(&self) -> Vec<Node> {
        todo!()
    }
    pub fn next(&self) -> Option<Node> {
        todo!()
    }
    pub fn next_all(&self) -> Vec<Node> {
        todo!()
    }
    pub fn prev(&self) -> Option<Node> {
        todo!()
    }
    pub fn prev_all(&self) -> Vec<Node> {
        todo!()
    }
    pub fn eq(&self, _i: usize) -> Node {
        todo!()
    }
    pub fn each<F>(&self, _f: F)
    where
        F: Fn(&Node),
    {
        todo!()
    }
}

// tree manipulation API
impl Node {
    pub fn attr(&mut self) {}
    pub fn replace(&mut self, pattern_str: &str, replacement_str: &str) -> &mut Self {
        let to_match = pattern::Pattern::new(pattern_str);
        let _to_replace = pattern::Pattern::new(replacement_str);
        if let Some(_node) = to_match.match_node(self) {
            todo!("change node content with replaced")
        } else {
            todo!()
        }
    }
    pub fn replace_by(&mut self) {}
    pub fn after(&mut self) {}
    pub fn before(&mut self) {}
    pub fn append(&mut self) {}
    pub fn prepend(&mut self) {}
    pub fn empty(&mut self) {}
    pub fn remove(&mut self) {}
    pub fn clone(&mut self) {}
}

// creational API
impl Semgrep {
    pub fn new<S: AsRef<str>>(source: S) -> Self {
        Self {
            root: Node::new(source.as_ref()),
        }
    }
    pub fn generate(_n: &Node) -> String {
        todo!()
    }
}

impl Deref for Semgrep {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}
impl DerefMut for Semgrep {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}

#[cfg(test)]
mod test {
    /*
    use super::*;
     #[test]
     fn test_replace() {
         let mut node = Semgrep::new("var a = 1;");
         node.replace("var $_$ = $_$", "let $_$ = $_$");
         let replaced = Semgrep::generate(&node);
         assert_eq!(replaced, "let a = 1");
    }
    */
}
