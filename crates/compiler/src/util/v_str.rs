//! There is still a lot we can optimize VStr
//! * instead of using &str, we can use intern to cache static attr name.
//! * we can also cache camelize/capitalize result.
//! * if VStr raw already satisfy StrOps, setting the ops flag is noop.
//! * interning/cache can be optional, e.g. Text Token can skip it at all.
use super::is_event_prop;
use bitflags::bitflags;
use std::{
    io::{self, Write},
    ops::Deref,
};

bitflags! {
    /// Represents idempotent string manipulation.
    // Idempotency is required since op is a bitflag.
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

impl StrOps {
    // ideally it should be str.satisfy(op) but adding a trait
    // to str is too much. Use passive voice.
    fn is_satisfied_by(&self, s: &str) -> bool {
        todo!()
    }
    fn write_ops<W: Write>(&self, s: &str, mut w: W) -> io::Result<()> {
        // TODO: add real impl
        w.write_all(s.as_bytes())
    }
}

/// A str for Vue compiler's internal modification.
/// Instead of returning a Cow<str>, StrOp is recorded in the VStr
/// and will be processed later in codegen phase.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
