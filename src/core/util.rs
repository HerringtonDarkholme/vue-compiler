use super::{
    parser::{Directive, DirectiveArg, ElemProp, Element},
    runtime_helper::RuntimeHelper,
    tokenizer::Attribute,
};
use bitflags::bitflags;
#[cfg(test)]
use serde::Serialize;
use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    ops::Deref,
};

pub fn non_whitespace(c: char) -> bool {
    !c.is_ascii_whitespace()
}

pub fn get_core_component(tag: &str) -> Option<RuntimeHelper> {
    use RuntimeHelper as RH;
    Some(match tag {
        "Teleport" | "teleport" => RH::TELEPORT,
        "Suspense" | "suspense" => RH::SUSPENSE,
        "KeepAlive" | "keep-alive" => RH::KEEP_ALIVE,
        "BaseTransition" | "base-transition" => RH::BASE_TRANSITION,
        _ => return None,
    })
}

pub fn is_core_component(tag: &str) -> bool {
    get_core_component(tag).is_some()
}

pub const fn yes(_: &str) -> bool {
    true
}
pub const fn no(_: &str) -> bool {
    false
}

pub trait PropPattern {
    fn is_match(&self, name: &str) -> bool;
}
impl PropPattern for &str {
    fn is_match(&self, name: &str) -> bool {
        name == *self
    }
}

impl<F> PropPattern for F
where
    F: Fn(&str) -> bool,
{
    fn is_match(&self, name: &str) -> bool {
        self(name)
    }
}

impl<const N: usize> PropPattern for [&'static str; N] {
    fn is_match(&self, name: &str) -> bool {
        self.contains(&name)
    }
}

type NameExp<'a> = Option<(&'a str, Option<VStr<'a>>)>;
pub trait PropMatcher<'a> {
    fn get_name_and_exp(prop: &ElemProp<'a>) -> NameExp<'a>;
    fn get_ref<'b>(prop: &'b ElemProp<'a>) -> &'b Self;
    fn take(prop: ElemProp<'a>) -> Self;
    fn is_match<P>(p: &ElemProp<'a>, pat: &P, allow_empty: bool) -> bool
    where
        P: PropPattern,
    {
        Self::get_name_and_exp(p).map_or(false, |(name, exp)| {
            pat.is_match(name) && (allow_empty || exp.map_or(false, |v| !v.is_empty()))
        })
    }
}

pub fn is_bind_key<'a>(arg: &Option<DirectiveArg<'a>>, name: &str) -> bool {
    get_bind_key(arg).map_or(false, |v| v == name)
}

fn get_bind_key<'a>(arg: &Option<DirectiveArg<'a>>) -> Option<&'a str> {
    if let DirectiveArg::Static(name) = arg.as_ref()? {
        Some(name)
    } else {
        None
    }
}

impl<'a> PropMatcher<'a> for ElemProp<'a> {
    fn get_name_and_exp(prop: &ElemProp<'a>) -> NameExp<'a> {
        match prop {
            ElemProp::Attr(Attribute { name, value, .. }) => {
                let exp = value.as_ref().map(|v| v.content);
                Some((name, exp))
            }
            ElemProp::Dir(dir @ Directive { name: "bind", .. }) => {
                let name = get_bind_key(&dir.argument)?;
                let exp = dir.expression.as_ref().map(|v| v.content);
                Some((name, exp))
            }
            _ => None,
        }
    }
    fn get_ref<'b>(prop: &'b ElemProp<'a>) -> &'b Self {
        prop
    }
    fn take(prop: ElemProp<'a>) -> Self {
        prop
    }
}

impl<'a> PropMatcher<'a> for Directive<'a> {
    fn get_name_and_exp(prop: &ElemProp<'a>) -> NameExp<'a> {
        if let ElemProp::Dir(Directive {
            name, expression, ..
        }) = prop
        {
            let exp = expression.as_ref().map(|v| v.content);
            Some((name, exp))
        } else {
            None
        }
    }
    fn get_ref<'b>(prop: &'b ElemProp<'a>) -> &'b Self {
        if let ElemProp::Dir(dir) = prop {
            return dir;
        }
        unreachable!("invalid call")
    }
    fn take(prop: ElemProp<'a>) -> Self {
        if let ElemProp::Dir(dir) = prop {
            return dir;
        }
        unreachable!("invalid call")
    }
}

pub struct PropFound<'a, E, M>
where
    E: Borrow<Element<'a>>,
    M: PropMatcher<'a>,
{
    elem: E,
    pos: usize,
    m: PhantomData<&'a M>,
}

impl<'a, E, M> PropFound<'a, E, M>
where
    E: Borrow<Element<'a>>,
    M: PropMatcher<'a>,
{
    fn new(elem: E, pos: usize) -> Option<Self> {
        Some(Self {
            elem,
            pos,
            m: PhantomData,
        })
    }
    pub fn get_ref(&self) -> &M {
        M::get_ref(&self.elem.borrow().properties[self.pos])
    }
}
// take is only available when access is mutable
impl<'a, E, M> PropFound<'a, E, M>
where
    E: BorrowMut<Element<'a>>,
    M: PropMatcher<'a>,
{
    pub fn take(mut self) -> M {
        M::take(self.elem.borrow_mut().properties.remove(self.pos))
    }
}

type DirFound<'a, E> = PropFound<'a, E, Directive<'a>>;

// sometimes mutable access to the element is not available so
// Borrow is used to refine PropFound so `take` is optional
pub fn dir_finder<'a, E, P>(elem: E, pat: P) -> PropFinder<'a, E, P, Directive<'a>>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat)
}

pub fn find_dir<'a, E, P>(elem: E, pat: P) -> Option<DirFound<'a, E>>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat).find()
}

pub struct PropFinder<'a, E, P, M = ElemProp<'a>>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
    M: PropMatcher<'a>,
{
    elem: E,
    pat: P,
    allow_empty: bool,
    m: PhantomData<&'a M>,
}

impl<'a, E, P, M> PropFinder<'a, E, P, M>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
    M: PropMatcher<'a>,
{
    fn new(elem: E, pat: P) -> Self {
        Self {
            elem,
            pat,
            allow_empty: false,
            m: PhantomData,
        }
    }
    fn is_match(&self, p: &ElemProp<'a>) -> bool {
        M::is_match(p, &self.pat, self.allow_empty)
    }
    pub fn find(self) -> Option<PropFound<'a, E, M>> {
        let pos = self
            .elem
            .borrow()
            .properties
            .iter()
            .position(|p| self.is_match(p))?;
        PropFound::new(self.elem, pos)
    }
    pub fn allow_empty(self, allow_empty: bool) -> Self {
        Self {
            allow_empty,
            ..self
        }
    }
}

impl<'a, P> PropFinder<'a, Element<'a>, P, ElemProp<'a>>
where
    P: PropPattern + Copy,
{
    pub fn find_all(self) -> impl Iterator<Item = Result<ElemProp<'a>, ElemProp<'a>>> {
        let PropFinder {
            elem,
            pat,
            allow_empty,
            ..
        } = self;
        elem.properties.into_iter().map(move |p| {
            if ElemProp::is_match(&p, &pat, allow_empty) {
                Ok(p)
            } else {
                Err(p)
            }
        })
    }
}

pub fn find_prop<'a, E, P>(elem: E, pat: P) -> Option<PropFound<'a, E, ElemProp<'a>>>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat).find()
}

pub fn prop_finder<'a, E, P>(elem: E, pat: P) -> PropFinder<'a, E, P>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat)
}

bitflags! {
    #[cfg_attr(test, derive(Serialize))]
    pub struct StrOps: u8 {
        const COMPRESS_WHITESPACE = 1 << 0;
        const DECODE_ENTITY       = 1 << 1;
        const CAMEL_CASE          = 1 << 2;
        const IS_ATTR             = 1 << 3;
        const HANDLER_KEY         = 1 << 4;
    }
}

/// A str for Vue compiler's internal modification.
/// Instead of returning a Cow<str>, StrOp is recorded in the VStr
/// and will be processed later in codegen phase.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Serialize))]
pub struct VStr<'a> {
    pub raw: &'a str,
    pub ops: StrOps,
}

impl<'a> VStr<'a> {
    // adjective is static method
    pub fn raw(raw: &'a str) -> Self {
        Self {
            raw,
            ops: StrOps::empty(),
        }
    }
    // verb is instance method
    pub fn decode(&mut self, is_attr: bool) -> &mut Self {
        let ops = if is_attr {
            StrOps::DECODE_ENTITY | StrOps::IS_ATTR
        } else {
            StrOps::DECODE_ENTITY
        };
        self.ops |= ops;
        self
    }
    pub fn camelize(&mut self) -> &mut Self {
        self.ops |= StrOps::CAMEL_CASE;
        self
    }
    pub fn compress_whitespace(&mut self) -> &mut Self {
        self.ops |= StrOps::COMPRESS_WHITESPACE;
        self
    }
    pub fn add_handler_key(&mut self) -> &mut Self {
        self.ops |= StrOps::HANDLER_KEY;
        self
    }
}

impl<'a> Deref for VStr<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::parser::test::mock_element;

    #[test]
    fn test_find_dir() {
        let e = mock_element("<p v-if=true/>");
        let found = find_dir(&e, "if");
        let found = found.expect("should found directive");
        assert_eq!(found.get_ref().name, "if");
        assert_eq!(e.properties.len(), 1);
    }

    #[test]
    fn test_find_dir_mut() {
        let mut e = mock_element("<p v-if=true/>");
        let found = find_dir(&mut e, "if");
        let found = found.expect("should found directive");
        assert_eq!(found.get_ref().name, "if");
        assert_eq!(found.take().name, "if");
        assert!(e.properties.is_empty());
    }

    #[test]
    fn test_find_empty_dir() {
        let e = mock_element("<p v-if=true v-for>");
        assert!(find_dir(&e, "if").is_some());
        assert!(find_dir(&e, "for").is_none());
        let found = dir_finder(&e, "for").allow_empty(true).find();
        assert!(found.is_some());
    }

    #[test]
    fn test_find_prop() {
        let mut e = mock_element("<p :name=foo name=bar/>");
        assert!(find_dir(&e, "name").is_none());
        assert!(find_dir(&e, "bind").is_some());
        // prop only looks at attr and v-bind
        assert!(find_prop(&e, "bind").is_none());
        find_prop(&mut e, "name").unwrap().take();
        assert!(find_prop(&e, "bind").is_none());
        find_prop(&mut e, "name").unwrap().take();
        assert!(find_prop(&e, "name").is_none());
    }

    #[test]
    fn find_prop_ignore_dynamic_bind() {
        let e = mock_element("<p :[name]=foo/>");
        assert!(find_dir(&e, "name").is_none());
        assert!(find_dir(&e, "bind").is_some());
        assert!(find_prop(&e, "name").is_none());
    }
}
