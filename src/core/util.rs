use super::{
    parser::{Directive, DirectiveArg, ElemProp, Element},
    runtime_helper::RuntimeHelper,
    tokenizer::Attribute,
};
use bitflags::bitflags;
#[cfg(test)]
use serde::Serialize;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
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
}

impl<'a> PropMatcher<'a> for ElemProp<'a> {
    fn get_name_and_exp(prop: &ElemProp<'a>) -> NameExp<'a> {
        match prop {
            ElemProp::Attr(Attribute { name, value, .. }) => {
                let exp = value.as_ref().map(|v| v.content);
                Some((name, exp))
            }
            ElemProp::Dir(dir @ Directive { name: "bind", .. }) => {
                let name = match dir.argument {
                    Some(DirectiveArg::Static(name)) => name,
                    _ => return None,
                };
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
    E: Deref<Target = Element<'a>>,
    M: PropMatcher<'a>,
{
    elem: E,
    pos: usize,
    m: PhantomData<M>,
}

impl<'a, E, M> PropFound<'a, E, M>
where
    E: Deref<Target = Element<'a>>,
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
        M::get_ref(&self.elem.properties[self.pos])
    }
}
// take is only available when access is mutable
impl<'a, E, M> PropFound<'a, E, M>
where
    E: DerefMut<Target = Element<'a>>,
    M: PropMatcher<'a>,
{
    pub fn take(mut self) -> M {
        M::take(self.elem.properties.remove(self.pos))
    }
}

type DirFound<'a, E> = PropFound<'a, E, Directive<'a>>;

// sometimes mutable access to the element is not available so
// Deref is used to override the PropFound and `take` is optional
pub fn dir_finder<'a, E, P>(elem: E, pat: P) -> PropFinder<'a, E, P, Directive<'a>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat)
}

pub fn find_dir<'a, E, P>(elem: E, pat: P) -> Option<DirFound<'a, E>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat).find()
}

pub struct PropFinder<'a, E, P, M = ElemProp<'a>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
    M: PropMatcher<'a>,
{
    elem: E,
    pat: P,
    allow_empty: bool,
    m: PhantomData<M>,
}

impl<'a, E, P, M> PropFinder<'a, E, P, M>
where
    E: Deref<Target = Element<'a>>,
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
        M::get_name_and_exp(p).map_or(false, |(name, exp)| {
            self.pat.is_match(name) && (self.allow_empty || exp.map_or(false, |v| !v.is_empty()))
        })
    }
    pub fn find(self) -> Option<PropFound<'a, E, M>> {
        let pos = self.elem.properties.iter().position(|p| self.is_match(p))?;
        PropFound::new(self.elem, pos)
    }
}

pub fn find_prop<'a, E, P>(elem: E, pat: P) -> Option<PropFound<'a, E, ElemProp<'a>>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat).find()
}

pub fn prop_finder<'a, E, P>(elem: E, pat: P) -> PropFinder<'a, E, P>
where
    E: Deref<Target = Element<'a>>,
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
}
