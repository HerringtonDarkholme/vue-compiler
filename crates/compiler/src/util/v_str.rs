//! There is still a lot we can optimize VStr
//! * instead of using &str, we can use intern to cache static attr name.
//! * we can also cache camelize/capitalize result.
//! * if VStr raw already satisfy StrOps, setting the ops flag is noop.
//! * interning/cache can be optional, e.g. Text Token can skip it at all.
use super::{is_event_prop, non_whitespace};
use bitflags::bitflags;
use std::{
    io::{self, Write},
    ops::Deref,
};

bitflags! {
    /// Represents idempotent string manipulation.
    // Idempotency is required since op is a bitflag.
    #[derive(Default)]
    pub struct StrOps: u16 {
        const COMPRESS_WHITESPACE = 1 << 0;
        const DECODE_ENTITY       = 1 << 1;
        const CAMEL_CASE          = 1 << 2;
        const PASCAL_CASE         = 1 << 3;
        const IS_ATTR             = 1 << 4;
        const HANDLER_KEY         = 1 << 5;
        const VALID_DIR           = 1 << 6;
        const VALID_COMP          = 1 << 7;
        const SELF_SUFFIX         = 1 << 8; // not idempotent but called only once
        const V_DIR_PREFIX        = 1 << 9;
        const JS_STRING           = 1 << 10;
        // TODO: add idempotent_ops and affine_ops. affine comes from
        // https://en.wikipedia.org/wiki/Substructural_type_system
    }
}

fn write_hyphenated<W: Write>(s: &str, mut w: W) -> io::Result<()> {
    // JS word boundary is `\w`: `[a-zA-Z0-9-]`.
    // https://javascript.info/regexp-boundary
    // str.replace(/\B([A-Z])/g, '-$1').toLowerCase()
    let mut is_boundary = true;
    for c in s.chars() {
        if !is_boundary && c.is_ascii_uppercase() {
            w.write_all(b"-")?;
            write!(w, "{}", c.to_ascii_lowercase())?;
            is_boundary = false;
        } else {
            write!(w, "{}", c)?;
            is_boundary = !c.is_ascii_alphanumeric() && c != '_';
        }
    }
    Ok(())
}

fn write_json_string<W: Write>(s: &str, w: &mut W) -> io::Result<()> {
    use json::codegen::{Generator, WriterGenerator};
    let mut gen = WriterGenerator::new(w);
    gen.write_string(s)
}

/// compress consecutive whitespaces into one.
fn write_compressed<W: Write>(mut s: &str, mut w: W) -> io::Result<()> {
    while let Some(p) = s.find(|c: char| c.is_ascii_whitespace()) {
        let (prev, after) = s.split_at(p);
        w.write_all(prev.as_bytes())?;
        w.write_all(b" ")?;
        if let Some(p) = after.find(non_whitespace) {
            s = after.split_at(p).1;
        } else {
            s = "";
        }
    }
    w.write_all(s.as_bytes())
}

/// decode html entity before writing.
fn write_decoded<W: Write>(s: &str, mut w: W) -> io::Result<()> {
    if !s.contains('&') {
        return w.write_all(s.as_bytes());
    }
    todo!()
}

impl StrOps {
    // ideally it should be str.satisfy(op) but adding a trait
    // to str is too much. Use passive voice.
    fn is_satisfied_by(&self, s: &str) -> bool {
        todo!()
    }
    fn write_ops<W: Write>(&self, s: &str, mut w: W) -> io::Result<()> {
        let flag_count = self.bits().count_ones();
        if flag_count == 0 {
            return w.write_all(s.as_bytes());
        }
        if flag_count == 1 {
            return Self::write_one_op(*self, s, w);
        }
        let mut src = s;
        let mut temp = vec![];
        let mut dest = vec![];
        for op in self.iter() {
            Self::write_one_op(op, src, &mut dest)?;
            std::mem::swap(&mut temp, &mut dest);
            dest.clear();
            src = std::str::from_utf8(&temp).expect("must be valid string");
        }
        w.write_all(src.as_bytes())
    }
    fn write_one_op<W: Write>(op: Self, s: &str, mut w: W) -> io::Result<()> {
        debug_assert!(op.bits().count_ones() == 1);
        match op {
            StrOps::COMPRESS_WHITESPACE => write_compressed(s, w),
            StrOps::DECODE_ENTITY => write_decoded(s, w),
            StrOps::JS_STRING => write_json_string(s, &mut w),
            StrOps::IS_ATTR => w.write_all(s.as_bytes()), // NOOP
            StrOps::SELF_SUFFIX => {
                w.write_all(s.as_bytes())?;
                w.write_all(b"__self")
            }
            StrOps::V_DIR_PREFIX => {
                w.write_all(b"v-")?;
                w.write_all(s.as_bytes())
            }
            _ => todo!("{:?} not implemented", op),
        }
    }
    fn iter(&self) -> StrOpIter {
        StrOpIter(*self)
    }
}

struct StrOpIter(StrOps);
impl Iterator for StrOpIter {
    type Item = StrOps;
    fn next(&mut self) -> Option<Self::Item> {
        let ops = &mut self.0;
        if ops.is_empty() {
            None
        } else {
            let bits = 1 << ops.bits().trailing_zeros();
            let r = StrOps { bits };
            ops.remove(r);
            Some(r)
        }
    }
}

/// A str for Vue compiler's internal modification.
/// Instead of returning a Cow<str>, StrOp is recorded in the VStr
/// and will be processed later in codegen phase.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VStr<'a> {
    pub raw: &'a str,
    pub ops: StrOps,
}

impl<'a> VStr<'a> {
    // adjective and is_xx for static method
    pub fn raw(raw: &'a str) -> Self {
        Self {
            raw,
            ops: StrOps::empty(),
        }
    }
    pub fn is_handler(s: &VStr) -> bool {
        if s.ops.contains(StrOps::HANDLER_KEY) {
            return true;
        }
        is_event_prop(s.raw)
    }
}
impl<'a> VStr<'a> {
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
    pub fn capitalize(&mut self) -> &mut Self {
        self.ops |= StrOps::PASCAL_CASE;
        self
    }
    pub fn compress_whitespace(&mut self) -> &mut Self {
        self.ops |= StrOps::COMPRESS_WHITESPACE;
        self
    }
    /// convert v-on arg to handler key: click -> onClick
    pub fn be_handler(&mut self) -> &mut Self {
        self.ops |= StrOps::HANDLER_KEY;
        self
    }
    /// add __self suffix for self referring component
    pub fn suffix_self(&mut self) -> &mut Self {
        self.ops |= StrOps::SELF_SUFFIX;
        self
    }
    /// convert into a valid asset id
    pub fn be_component(&mut self) -> &mut Self {
        self.ops |= StrOps::VALID_COMP;
        self
    }
    pub fn be_directive(&mut self) -> &mut Self {
        self.ops |= StrOps::VALID_DIR;
        self
    }
    /// convert into a valid asset id
    pub fn prefix_v_dir(&mut self) -> &mut Self {
        self.ops |= StrOps::V_DIR_PREFIX;
        self
    }
    pub fn be_js_str(&mut self) -> &mut Self {
        self.ops |= StrOps::JS_STRING;
        self
    }
    pub fn into_string(self) -> String {
        let mut ret = vec![];
        self.write_to(&mut ret).expect("string should never fail");
        String::from_utf8(ret).expect("vstr should write valid utf8")
    }

    pub fn write_to<W: Write>(&self, w: W) -> io::Result<()> {
        self.ops.write_ops(self.raw, w)
    }
}

impl<'a> Deref for VStr<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> From<&'a str> for VStr<'a> {
    fn from(s: &'a str) -> Self {
        VStr::raw(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: proptest can test invariant
    #[test]
    fn test_str_ops_iter() {
        let a = StrOps::all();
        let v: Vec<_> = a.iter().collect();
        assert_eq!(v.len() as u32, a.bits().count_ones());
        assert!(v.iter().all(|op| op.bits().count_ones() == 1));
        let a = StrOps::empty();
        let v = a.iter().count();
        assert_eq!(v, 0);
        let a = StrOps::V_DIR_PREFIX | StrOps::VALID_COMP;
        let v: Vec<_> = a.iter().collect();
        assert_eq!(v[0], StrOps::VALID_COMP);
        assert_eq!(v[1], StrOps::V_DIR_PREFIX);
        assert_eq!(v.len(), 2);
    }

    fn write_string(ops: StrOps, s: &str) -> String {
        let mut w = vec![];
        ops.write_ops(s, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }

    #[test]
    fn test_str_ops_write() {
        let src = "test";
        let cases = [
            (StrOps::empty(), "test"),
            (StrOps::V_DIR_PREFIX, "v-test"),
            (StrOps::SELF_SUFFIX, "test__self"),
            (StrOps::JS_STRING, stringify!("test")),
            (StrOps::SELF_SUFFIX | StrOps::V_DIR_PREFIX, "v-test__self"),
        ];
        for (ops, expect) in cases {
            let origin = ops;
            assert_eq!(write_string(ops, src), expect);
            assert_eq!(ops, origin);
        }
    }
}
