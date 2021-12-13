use std::ops::{Deref, DerefMut};
pub struct Semgrep {
    root: Node,
}

pub struct Node {}

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
    pub fn siblings(&self) -> Vec<Node> {
        todo!()
    }
    pub fn next(&self) -> Node {
        todo!()
    }
    pub fn next_all(&self) -> Vec<Node> {
        todo!()
    }
    pub fn prev(&self) -> Node {
        todo!()
    }
    pub fn prev_all(&self) -> Vec<Node> {
        todo!()
    }
    pub fn eq(&self, _i: usize) -> Node {
        todo!()
    }
    pub fn each<F>(&self, f: F)
    where
        F: Fn(&Node),
    {
    }
}

// tree manipulation API
impl Node {
    pub fn attr(&mut self) {}
    pub fn replace(&mut self, pattern: &str, replacement: &str) -> &mut Self {
        todo!()
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
            root: Self::parse(source.as_ref()),
        }
    }
    pub fn parse(_s: &str) -> Node {
        todo!()
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
