use super::{
    super::error::CompilationErrorKind as ErrorKind, AstNode, BaseConvertInfo, BaseConverter as BC,
    BaseIR, CompilationError, Directive, Element, IRNode, IfBranch, IfNodeIR,
};
use crate::core::{
    converter::{CoreConverter, JsExpr},
    util::find_dir,
};
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

    fn next_standalone(&mut self) -> Option<PreGroup<'a>> {
        debug_assert!(self
            .inner
            .peek()
            .map_or(true, |n| { n.get_element().is_none() }));
        self.inner.next().map(PreGroup::StandAlone)
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
            } else if let AstNode::Text(s) = n {
                if s.is_all_whitespace() {
                    // skip whitespace
                    self.next().unwrap();
                } else {
                    // break if text is not whitespaces
                    break;
                }
            } else if matches!(n, &AstNode::Comment(_)) {
                // ignore comments for now. #3619
                return self.next_standalone();
            } else {
                break;
            }
        }
        // vec emptied or next element has no v-if
        // first, flush preceding group
        self.flush_group().or_else(|| {
            // if no group, consume next standalone element if available
            self.next_standalone()
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}

/// pre group adjacent elements with v-if
// using generator here will be super cool
pub fn pre_group_v_if(children: Vec<AstNode>) -> impl Iterator<Item = PreGroup> {
    PreGroupIter::new(children)
}

pub fn convert_if<'a>(c: &BC, nodes: Vec<Element<'a>>, key: usize) -> BaseIR<'a> {
    let branches = nodes
        .into_iter()
        .map(|n| convert_if_branch(c, n, key))
        .collect();
    IRNode::If(IfNodeIR { branches, info: () })
}

fn convert_if_branch<'a>(
    c: &BC,
    mut e: Element<'a>,
    start_key: usize,
) -> IfBranch<BaseConvertInfo<'a>> {
    let dir = find_dir(&mut e, ["if", "else-if", "else"])
        .expect("the element must have v-if directives")
        .take();
    let condition = convert_if_condition(c, dir);
    IfBranch {
        children: vec![],
        condition,
        info: start_key,
    }
}
fn convert_if_condition<'a>(c: &BC, dir: Directive<'a>) -> Option<JsExpr<'a>> {
    if dir.name != "else" {
        if let Some(err) = dir.check_empty_expr(ErrorKind::VIfNoExpression) {
            c.emit_error(err);
            return Some(JsExpr::Lit("true"));
        }
    } else if let Some(expr) = dir.expression {
        let error =
            CompilationError::new(ErrorKind::UnexpectedDirExpression).with_location(expr.location);
        c.emit_error(error);
        return None;
    }
    dir.expression.map(|v| JsExpr::Simple(v.content))
}

#[cfg(test)]
mod test {
    fn test() {
        let cases = vec![
            r#"
<p v-if="false">a</p>
<p v-else v-if="true">b</p>
<p v-else>c</p>"#,
            r#"<p v-if="123"/><p v-else="33"/>"#,
            r#"<p v-if/>"#,
        ];
    }
}
