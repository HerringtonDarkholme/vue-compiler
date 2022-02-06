use std::ops::{Deref, DerefMut};
use std::rc::Rc;

mod js_parser;
mod language;
mod matcher;
mod meta_var;
mod pattern;
mod rule;

pub use pattern::Pattern;

pub struct Semgrep {
    root: Root,
}

pub struct Root {
    inner: js_parser::Tree,
    source: Rc<String>,
}

impl Root {
    fn new(src: &str) -> Self {
        Self {
            inner: js_parser::parse(src),
            source: Rc::new(src.into()),
        }
    }
    pub fn root(&self) -> Node {
        Node {
            inner: self.inner.root_node(),
            source: &self.source,
        }
    }
}

// the lifetime r represents root
#[derive(Clone, Copy)]
pub struct Node<'r> {
    inner: tree_sitter::Node<'r>,
    source: &'r str,
}
type NodeKind = u16;

struct NodeWalker<'tree> {
    cursor: tree_sitter::TreeCursor<'tree>,
    source: &'tree str,
    initiated: bool,
}

impl<'tree> Iterator for NodeWalker<'tree> {
    type Item = Node<'tree>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.initiated {
            if self.cursor.goto_first_child() {
                self.initiated = true;
                Some(Node {
                    inner: self.cursor.node(),
                    source: self.source,
                })
            } else {
                None
            }
        } else if self.cursor.goto_next_sibling() {
            Some(Node {
                inner: self.cursor.node(),
                source: self.source,
            })
        } else {
            None
        }
    }
}

// internal API
impl<'r> Node<'r> {
    fn is_leaf(&self) -> bool {
        self.inner.child_count() == 0
    }
    fn kind_id(&self) -> NodeKind {
        self.inner.kind_id()
    }
    pub fn text(&self) -> &str {
        self.inner
            .utf8_text(self.source.as_bytes())
            .expect("invalid source text encoding")
    }

    pub fn children(&self) -> impl Iterator<Item = Node<'r>> {
        NodeWalker {
            cursor: self.inner.walk(),
            source: self.source,
            initiated: false,
        }
    }
}

// tree traversal API
impl<'r> Node<'r> {
    #[must_use]
    pub fn find(&self) -> Node<'r> {
        todo!()
    }
    // should we provide parent?
    #[must_use]
    pub fn parent(&self) -> Node<'r> {
        todo!()
    }
    #[must_use]
    pub fn ancestors(&self) -> Vec<Node<'r>> {
        todo!()
    }
    #[must_use]
    pub fn next(&self) -> Option<Node<'r>> {
        todo!()
    }
    #[must_use]
    pub fn next_all(&self) -> Vec<Node<'r>> {
        todo!()
    }
    #[must_use]
    pub fn prev(&self) -> Option<Node<'r>> {
        todo!()
    }
    #[must_use]
    pub fn prev_all(&self) -> Vec<Node<'r>> {
        todo!()
    }
    #[must_use]
    pub fn eq(&self, _i: usize) -> Node<'r> {
        todo!()
    }
    pub fn each<F>(&self, _f: F)
    where
        F: Fn(&Node<'r>),
    {
        todo!()
    }
}

// r manipulation API
impl<'r> Node<'r> {
    pub fn attr(&mut self) {}
    pub fn replace(&mut self, pattern_str: &str, replacement_str: &str) -> &mut Self {
        let to_match = pattern::Pattern::new(pattern_str);
        let _to_replace = pattern::Pattern::new(replacement_str);
        todo!()
        // if let Some(_node) = to_match.match_node(self) {
        //     todo!("change node content with replaced")
        // } else {
        //     todo!()
        // }
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
            root: Root::new(source.as_ref()),
        }
    }
    pub fn generate(_n: &Node) -> String {
        todo!()
    }
}

impl Deref for Semgrep {
    type Target = Root;
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
