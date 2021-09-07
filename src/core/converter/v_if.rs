use super::{AstNode, CoreConvertInfo, Element, IRNode, IfBranch, IfNodeIR};
use crate::core::util::find_dir;
use std::{iter::Peekable, vec::IntoIter};

pub enum PreGroup<'a> {
    VIfGroup(Vec<Element<'a>>),
    StandAlone(AstNode<'a>),
}

struct PreGroupIter<'a> {
    inner: Peekable<IntoIter<AstNode<'a>>>,
    group: Vec<Element<'a>>,
}

impl<'a> PreGroupIter<'a> {
    fn new(children: Vec<AstNode<'a>>) -> Self {
        let len = children.len();
        Self {
            inner: children.into_iter().peekable(),
            group: Vec::with_capacity(len),
        }
    }
    fn flush_group(&mut self) -> Option<PreGroup<'a>> {
        if self.group.is_empty() {
            None
        } else {
            let group = self.group.drain(..).collect();
            Some(PreGroup::VIfGroup(group))
        }
    }
}
impl<'a> Iterator for PreGroupIter<'a> {
    type Item = PreGroup<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(n) = self.inner.peek() {
            // collect elements while they have v-if
            let found_v_if = n
                .get_element()
                .and_then(|e| find_dir(e, ["if", "else-if", "else"]))
                .is_some();
            if found_v_if {
                let n = self.inner.next().unwrap(); // must next to advance
                let e = n.into_element().unwrap();
                self.group.push(e);
            } else {
                // TODO: add comment and empty text handling
                break;
            }
        }
        // vec emptied or next element has no v-if
        // first, flush preceding group
        self.flush_group().or_else(|| {
            // if no group, consume next standalone element if available
            self.inner.next().map(|n| PreGroup::StandAlone(n))
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}

/// pre group adjacent elements with v-if
// using generator here will be super cool
pub fn pre_group_v_if<'a>(children: Vec<AstNode<'a>>) -> impl Iterator<Item = PreGroup<'a>> {
    PreGroupIter::new(children)
}

pub fn convert_if(nodes: Vec<AstNode>, key: usize) -> IRNode<CoreConvertInfo> {
    let branches = nodes
        .into_iter()
        .map(|n| convert_if_branch(n, key))
        .collect();
    IRNode::If(IfNodeIR { branches, info: () })
}

fn convert_if_branch(node: AstNode, start_key: usize) -> IfBranch<CoreConvertInfo> {
    IfBranch {
        children: vec![],
        condition: todo!(),
    }
}

#[cfg(test)]
mod test {
    fn test() {
        let cases = vec![
            r#"
<p v-if="false">a</p>
<p v-else v-if="true">b</p>
<p v-else>c</p>"#,
        ];
    }
}
