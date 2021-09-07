use super::{
    parser::{Directive, Element},
    runtime_helper::RuntimeHelper,
};

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

pub struct FoundDir<'s, 'a> {
    dirs: &'s mut Vec<Directive<'a>>,
    pos: usize,
}
impl<'s, 'a> FoundDir<'s, 'a> {
    pub fn take(self) -> Directive<'a> {
        self.dirs.remove(self.pos)
    }
    pub fn as_ref(&self) -> &Directive<'a> {
        &self.dirs[self.pos]
    }
}

pub fn find_dir<'s, 'a, P>(e: &'s mut Element<'a>, pattern: P) -> Option<FoundDir<'s, 'a>>
where
    P: DirPattern,
{
    let pos = e
        .directives
        .iter()
        .position(|dir| pattern.is_match(dir.name))?;
    Some(FoundDir {
        pos,
        dirs: &mut e.directives,
    })
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
        let mut e = mock_element(dir);
        let found = find_dir(&mut e, "if");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.as_ref().name, "if");
        assert_eq!(found.take().name, "if");
        assert!(e.directives.is_empty());
    }
}
