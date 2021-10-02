use super::{
    super::parser::ElemProp, AstNode, BaseConvertInfo, BaseConverter as BC, BaseIR,
    CompilationError, Directive, Element, IRNode, IfBranch, IfNodeIR,
};
use crate::{
    converter::{CoreConverter, JsExpr as Js},
    error::CompilationErrorKind as ErrorKind,
    scanner::Attribute,
    util::{find_dir_empty, find_prop, VStr},
};
use rustc_hash::FxHashSet;
use std::{iter::Peekable, vec::IntoIter};

// TODO: reduce vec allocation by using Drain iter
// but using drain need GAT
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
        // either iter is empty or has no if/else/else-if
        debug_assert!(self
            .inner
            .peek()
            .and_then(|n| n.get_element())
            .and_then(|e| find_dir_empty(e, ["if", "else", "else-if"]))
            .is_none());
        self.inner.next().map(PreGroup::StandAlone)
    }
}
impl<'a> Iterator for PreGroupIter<'a> {
    type Item = PreGroup<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(n) = self.inner.peek() {
            // group elements if they have v-if/v-else
            let found = n
                .get_element()
                .and_then(|e| find_dir_empty(e, ["if", "else-if", "else"]));
            if let Some(d) = found {
                // separate v-if into different groups
                if d.get_ref().name == "if" && !self.group.is_empty() {
                    return self.flush_group();
                }
                let n = self.inner.next().unwrap(); // must next to advance
                self.group.push(n.into_element());
            } else if let AstNode::Text(s) = n {
                if self.group.is_empty() || !s.is_all_whitespace() {
                    // break if text is not whitespaces
                    // or no preceding v-if
                    break;
                }
                // skip whitespace when v-if precedes
                self.inner.next().unwrap();
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

/// key is Vue-generated default key based on the number of sibling v-if.
pub fn convert_if<'a>(c: &BC<'a>, elems: Vec<Element<'a>>, key: usize) -> BaseIR<'a> {
    debug_assert!(!elems.is_empty());
    check_v_if_group(c, &elems);
    let branches: Vec<_> = elems
        .into_iter()
        .enumerate()
        .map(|(i, n)| convert_if_branch(c, n, key + i))
        .collect();
    IRNode::If(IfNodeIR { branches })
}

pub fn report_dangling_else<'a>(c: &BC<'a>, elem: &Element<'a>) {
    debug_assert!(find_dir_empty(elem, "if").is_none());
    let loc = find_dir_empty(elem, ["else-if", "else"])
        .expect("must have other v-if dir")
        .get_ref()
        .location
        .clone();
    let error = CompilationError::new(ErrorKind::VElseNoAdjacentIf).with_location(loc);
    c.emit_error(error);
}

fn check_duplicate_key<'a>(
    c: &BC<'a>,
    prop: &ElemProp<'a>,
    dirs: &mut FxHashSet<VStr<'a>>,
    attrs: &mut FxHashSet<VStr<'a>>,
) {
    match prop {
        ElemProp::Dir(Directive {
            expression: Some(v),
            ..
        }) => {
            if dirs.contains(&v.content) {
                let error =
                    CompilationError::new(ErrorKind::VIfSameKey).with_location(v.location.clone());
                c.emit_error(error);
            } else {
                dirs.insert(v.content);
            }
        }
        ElemProp::Attr(Attribute { value: Some(v), .. }) => {
            if attrs.contains(&v.content) {
                let error =
                    CompilationError::new(ErrorKind::VIfSameKey).with_location(v.location.clone());
                c.emit_error(error);
            } else {
                attrs.insert(v.content);
            }
        }
        _ => (),
    }
}

fn check_v_if_group<'a>(c: &BC<'a>, elems: &[Element<'a>]) {
    // 1. check dangling else
    if find_dir_empty(&elems[0], "if").is_none() {
        report_dangling_else(c, &elems[0]);
    }
    if !c.option.is_dev {
        return;
    }
    // 2. check duplicate v-if key in dev build
    let mut dirs = FxHashSet::default();
    let mut attrs = FxHashSet::default();
    let mut has_else = false;
    for child in elems {
        if has_else {
            report_dangling_else(c, child);
            continue;
        }
        let prop = find_prop(child, "key");
        if let Some(prop) = prop {
            check_duplicate_key(c, prop.get_ref(), &mut dirs, &mut attrs);
        }
        has_else = find_dir_empty(child, ["else"]).is_some();
    }
}

fn convert_if_branch<'a>(
    c: &BC<'a>,
    mut e: Element<'a>,
    key: usize,
) -> IfBranch<BaseConvertInfo<'a>> {
    let dir = find_dir_empty(&mut e, ["if", "else-if", "else"])
        .expect("the element must have v-if directives")
        .take();
    report_duplicate_v_if(c, &mut e);
    let condition = convert_if_condition(c, dir);
    IfBranch {
        child: Box::new(c.dispatch_element(e)),
        condition,
        info: key,
    }
}
fn convert_if_condition<'a>(c: &BC<'a>, dir: Directive<'a>) -> Option<Js<'a>> {
    if dir.name != "else" {
        if let Some(err) = dir.check_empty_expr(ErrorKind::VIfNoExpression) {
            c.emit_error(err);
            return Some(Js::Src("true"));
        }
    } else if let Some(expr) = dir.expression {
        let error =
            CompilationError::new(ErrorKind::UnexpectedDirExpression).with_location(expr.location);
        c.emit_error(error);
        return None;
    }
    dir.expression.map(|v| Js::simple(v.content))
}
fn report_duplicate_v_if<'a>(c: &BC<'a>, e: &mut Element<'a>) {
    // https://stackoverflow.com/a/48144226/2198656
    while let Some(found) = find_dir_empty(&mut *e, ["if", "else-if", "else"]) {
        let dir = found.take();
        let error = CompilationError::new(ErrorKind::VIfDuplicateDir).with_location(dir.location);
        c.emit_error(error);
    }
}

#[cfg(test)]
mod test {
    use super::super::test::*;
    use super::*;
    use crate::cast;

    fn test_no_panic() {
        let cases = [
            r#"
<p v-if="false">a</p>
<p v-else v-if="true">b</p>
<p v-else>c</p>"#,
            r#"<p v-if="123"/><p v-else="33"/>"#,
            r#"<p v-if/>"#,
            r#"<p v-if="1"/><p v-else-if="2"/><comp v-else/>"#, // key = 1, 2, 3
        ];
        for case in cases {
            base_convert(case);
        }
    }

    #[test]
    fn test_v_if() {
        let body = base_convert("<p v-if='true'/>").body;
        assert_eq!(body.len(), 1);
        let v_if = cast!(&body[0], IRNode::If);
        assert_eq!(v_if.branches.len(), 1);
        let condition = v_if.branches[0].condition.as_ref().unwrap();
        let cond = cast!(condition, Js::Simple);
        assert_eq!(cond.into_string(), "true");
    }
}
