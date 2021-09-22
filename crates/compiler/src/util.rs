use super::{
    converter::{BaseConvertInfo, VNodeIR},
    flags::RuntimeHelper,
    parser::{Directive, DirectiveArg, ElemProp, Element},
    tokenizer::Attribute,
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::UnsafeCell,
    marker::PhantomData,
    ops::Deref,
};

mod v_str;
pub use v_str::VStr;

pub fn non_whitespace(c: char) -> bool {
    !c.is_ascii_whitespace()
}

pub fn get_core_component(tag: &str) -> Option<RuntimeHelper> {
    use RuntimeHelper as RH;
    Some(match tag {
        "Teleport" | "teleport" => RH::Teleport,
        "Suspense" | "suspense" => RH::Suspense,
        "KeepAlive" | "keep-alive" => RH::KeepAlive,
        "BaseTransition" | "base-transition" => RH::BaseTransition,
        _ => return None,
    })
}

pub fn is_core_component(tag: &str) -> bool {
    get_core_component(tag).is_some()
}

fn is_event_prop(prop: &str) -> bool {
    let bytes = prop.as_bytes();
    // equivalent to /^on[^a-z]/
    bytes.len() > 2 && bytes.starts_with(b"on") && !bytes[3].is_ascii_lowercase()
}

pub fn is_mergeable_prop(prop: &str) -> bool {
    prop == "class" || prop == "style" || is_event_prop(prop)
}

pub fn is_simple_identifier(s: VStr) -> bool {
    let is_ident = |c: char| c == '$' || c == '_' || c.is_ascii_alphanumeric();
    let raw = s.raw;
    raw.chars().all(is_ident) && !raw.starts_with(|c: char| c.is_ascii_digit())
}

macro_rules! make_list {
    ( $($id: ident),* ) => {
        &[
            $(stringify!($id)),*
        ]
    }
}

// use simple contains for small str array
// benchmark shows linear scan takes at most 10ns
// while phf or bsearch takes 30ns
const ALLOWED_GLOBALS: &[&str] = make_list!(
    Infinity,
    undefined,
    NaN,
    isFinite,
    isNaN,
    parseFloat,
    parseInt,
    decodeURI,
    decodeURIComponent,
    encodeURI,
    encodeURIComponent,
    Math,
    Number,
    Date,
    Array,
    Object,
    Boolean,
    String,
    RegExp,
    Map,
    Set,
    JSON,
    Intl,
    BigInt
);
pub fn is_global_allow_listed(s: &str) -> bool {
    ALLOWED_GLOBALS.contains(&s)
}

// https://github.com/vuejs/rfcs/blob/master/active-rfcs/0008-render-function-api-change.md#special-reserved-props
const RESERVED: &[&str] = make_list!(
    key,
    ref,
    onVnodeMounted,
    onVnodeUpdated,
    onVnodeUnmounted,
    onVnodeBeforeMount,
    onVnodeBeforeUpdate,
    onVnodeBeforeUnmount
);

#[inline]
pub fn is_reserved_prop(tag: &str) -> bool {
    RESERVED.contains(&tag)
}

pub fn is_component_tag(tag: &str) -> bool {
    tag == "component" || tag == "Component"
}

pub const fn yes(_: &str) -> bool {
    true
}
pub const fn no(_: &str) -> bool {
    false
}

pub fn get_vnode_call_helper(v: &VNodeIR<BaseConvertInfo>) -> RuntimeHelper {
    use RuntimeHelper as RH;
    if v.is_block {
        return if v.is_component {
            RH::CreateBlock
        } else {
            RH::CreateElementBlock
        };
    }
    if v.is_component {
        RH::CreateVNode
    } else {
        RH::CreateElementVNode
    }
}

pub trait PropPattern {
    fn matches(&self, name: &str) -> bool;
}
impl PropPattern for &str {
    fn matches(&self, name: &str) -> bool {
        name == *self
    }
}

impl<F> PropPattern for F
where
    F: Fn(&str) -> bool,
{
    fn matches(&self, name: &str) -> bool {
        self(name)
    }
}

impl<const N: usize> PropPattern for [&'static str; N] {
    fn matches(&self, name: &str) -> bool {
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
            pat.matches(name) && (allow_empty || !exp.map_or(true, |v| v.is_empty()))
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
        // TODO: avoid O(n) behavior
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

pub fn find_dir_empty<'a, E, P>(elem: E, pat: P) -> Option<DirFound<'a, E>>
where
    E: Borrow<Element<'a>>,
    P: PropPattern,
{
    PropFinder::new(elem, pat).allow_empty().find()
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
    filter: fn(&ElemProp<'a>) -> bool,
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
            filter: |_| true,
            m: PhantomData,
        }
    }
    fn is_match(&self, p: &ElemProp<'a>) -> bool {
        M::is_match(p, &self.pat, self.allow_empty)
    }
    pub fn dynamic_only(self) -> Self {
        Self {
            filter: |p| matches!(p, ElemProp::Dir(..)),
            ..self
        }
    }
    pub fn find(self) -> Option<PropFound<'a, E, M>> {
        let pos = self
            .elem
            .borrow()
            .properties
            .iter()
            .position(|p| self.is_match(p) && (self.filter)(p))?;
        PropFound::new(self.elem, pos)
    }
    pub fn allow_empty(self) -> Self {
        Self {
            allow_empty: true,
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

// since std::once::Lazy is not stable
// it is not thread safe, not Sync.
// it is Send if F and T is Send
pub struct Lazy<T, F = fn() -> T>(UnsafeCell<Result<T, Option<F>>>)
where
    F: FnOnce() -> T;

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub fn new(f: F) -> Self {
        Self(UnsafeCell::new(Err(Some(f))))
    }
}

impl<T, F> Deref for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let m = unsafe { &mut *self.0.get() };
        let f = match m {
            Ok(t) => return t,
            Err(f) => f,
        };
        *m = Ok(f.take().unwrap()());
        match m {
            Ok(t) => t,
            _ => panic!("unwrap Ok"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::test::mock_element;

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
        let found = dir_finder(&e, "for").allow_empty().find();
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
    #[test]
    fn find_dynamic_only_prop() {
        let e = mock_element("<p name=foo/>");
        assert!(prop_finder(&e, "name").dynamic_only().find().is_none());
        let e = mock_element("<p v-bind:name=foo/>");
        assert!(prop_finder(&e, "name").dynamic_only().find().is_some());
        let e = mock_element("<p :name=foo/>");
        assert!(prop_finder(&e, "name").dynamic_only().find().is_some());
        let e = mock_element("<p :[name]=foo/>");
        assert!(prop_finder(&e, "name").dynamic_only().find().is_none());
    }
    #[test]
    fn prop_find_all() {
        let e = mock_element("<p :name=foo name=bar :[name]=baz/>");
        let a: Vec<_> = prop_finder(e, "name").find_all().collect();
        assert_eq!(a.len(), 3);
        assert!(a[0].is_ok());
        assert!(a[1].is_ok());
        assert!(a[2].is_err());
    }

    #[test]
    fn layman_lazy() {
        let mut test = 0;
        let l = Lazy::new(|| {
            test += 1;
            (0..=100).sum::<i32>()
        });
        assert_eq!(*l, 5050);
        assert_eq!(*l, 5050);
        assert_eq!(test, 1);
    }
}
