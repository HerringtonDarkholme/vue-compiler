use super::{
    parser::{Directive, ElemProp, Element},
    runtime_helper::RuntimeHelper,
    tokenizer::Attribute,
};
use bitflags::bitflags;
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

pub trait PropMatcher<'a> {
    fn get_name(prop: &ElemProp<'a>) -> Option<&'a str>;
    fn get_ref<'b>(prop: &'b ElemProp<'a>) -> &'b Self;
    fn take(prop: ElemProp<'a>) -> Self;
}

impl<'a> PropMatcher<'a> for ElemProp<'a> {
    fn get_name(prop: &ElemProp<'a>) -> Option<&'a str> {
        match prop {
            ElemProp::Attr(Attribute { name, .. }) => Some(name),
            ElemProp::Dir(Directive { name, .. }) => Some(name),
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
    fn get_name(prop: &ElemProp<'a>) -> Option<&'a str> {
        if let ElemProp::Dir(Directive { name, .. }) = prop {
            Some(name)
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

pub struct PropFound<'a, E, M = ElemProp<'a>>
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
    fn new<P: PropPattern>(elem: E, pat: P) -> Option<Self> {
        let pos = elem
            .properties
            .iter()
            .position(|p| M::get_name(p).map_or(false, |n| pat.is_match(n)))?;
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
type AttrFound<'a, E> = PropFound<'a, E, Attribute<'a>>;

// sometimes mutable access to the element is not available so
// Deref is used to override the PropFound and `take` is optional
pub fn find_dir<'a, E, P>(e: E, pattern: P) -> Option<DirFound<'a, E>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
{
    PropFound::new(e, pattern)
}

pub fn find_prop<'a, E, P>(e: E, pattern: P) -> Option<PropFound<'a, E>>
where
    E: Deref<Target = Element<'a>>,
    P: PropPattern,
{
    PropFound::new(e, pattern)
}

bitflags! {
    pub struct StrOps: u8 {
        const COMPRESS_WHITESPACE = 1 << 0;
        const DECODE_ENTITY       = 1 << 1;
        const CAMEL_CASE          = 1 << 2;
        const IS_ATTR             = 1 << 3;
    }
}

/// A str for Vue compiler's internal modification.
/// Instead of returning a Cow<str>, StrOp is recorded in the VStr
/// and will be processed later in codegen phase.
#[derive(Debug, Clone, Copy)]
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
    use crate::core::Namespace;

    fn mock_element(dir: Directive) -> Element {
        Element {
            tag_name: "div",
            namespace: Namespace::Html,
            properties: vec![ElemProp::Dir(dir)],
            children: vec![],
            location: Default::default(),
        }
    }
    fn mock_directive(name: &str) -> Directive {
        Directive {
            name,
            ..Default::default()
        }
    }

    #[test]
    fn test_find_dir() {
        let dir = mock_directive("if");
        let e = mock_element(dir);
        let found = find_dir(&e, "if");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.get_ref().name, "if");
        assert_eq!(e.directives.len(), 1);
    }

    #[test]
    fn test_find_dir_mut() {
        let dir = mock_directive("if");
        let mut e = mock_element(dir);
        let found = find_dir(&mut e, "if");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.get_ref().name, "if");
        assert_eq!(found.take().name, "if");
        assert!(e.directives.is_empty());
    }
}
