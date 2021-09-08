use super::{
    parser::{Directive, Element},
    runtime_helper::RuntimeHelper,
};
use bitflags::bitflags;
use std::ops::{Deref, DerefMut};

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

pub trait DirPattern {
    fn is_match(&self, name: &str) -> bool;
}
impl DirPattern for &str {
    fn is_match(&self, name: &str) -> bool {
        name == *self
    }
}

impl<const N: usize> DirPattern for [&'static str; N] {
    fn is_match(&self, name: &str) -> bool {
        self.contains(&name)
    }
}

pub struct DirFound<'a, E>
where
    E: Deref<Target = Element<'a>>,
{
    elem: E,
    pos: usize,
}
impl<'a, E> DirFound<'a, E>
where
    E: Deref<Target = Element<'a>>,
{
    pub fn get_ref(&self) -> &Directive<'a> {
        &self.elem.directives[self.pos]
    }
}
// take is only available when access is mutable
impl<'a, E> DirFound<'a, E>
where
    E: DerefMut<Target = Element<'a>>,
{
    pub fn take(mut self) -> Directive<'a> {
        self.elem.directives.remove(self.pos)
    }
}

// sometimes mutable access to the element is not available so
// Deref is used to override the DirFound and `take` is optional
pub fn find_dir<'a, E, P>(e: E, pattern: P) -> Option<DirFound<'a, E>>
where
    E: Deref<Target = Element<'a>>,
    P: DirPattern,
{
    let pos = e
        .directives
        .iter()
        .position(|dir| pattern.is_match(dir.name))?;
    Some(DirFound { pos, elem: e })
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
    pub fn raw(raw: &'a str) -> Self {
        Self {
            raw,
            ops: StrOps::empty(),
        }
    }
    pub fn decode(self, is_attr: bool) -> Self {
        let new_flag = if is_attr {
            StrOps::DECODE_ENTITY | StrOps::IS_ATTR
        } else {
            StrOps::DECODE_ENTITY
        };
        Self {
            ops: self.ops | new_flag,
            raw: self.raw,
        }
    }
    pub fn camel(self) -> Self {
        Self {
            ops: self.ops | StrOps::CAMEL_CASE,
            raw: self.raw,
        }
    }
    pub fn compress_whitespace(&mut self) {
        self.ops |= StrOps::COMPRESS_WHITESPACE;
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
            attributes: vec![],
            directives: vec![dir],
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
