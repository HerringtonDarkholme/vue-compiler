//! There is still a lot we can optimize VStr
//! instead of using &str, we can use intern to cache static attr name
//! we can also cache camelize/capitalize result
use bitflags::bitflags;
#[cfg(test)]
use serde::Serialize;
use std::ops::Deref;
bitflags! {
    #[cfg_attr(test, derive(Serialize))]
    pub struct StrOps: u8 {
        const COMPRESS_WHITESPACE = 1 << 0;
        const DECODE_ENTITY       = 1 << 1;
        const CAMEL_CASE          = 1 << 2;
        const PASCAL_CASE         = 1 << 3;
        const IS_ATTR             = 1 << 4;
        const HANDLER_KEY         = 1 << 5;
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
    pub fn capitalize(&mut self) -> &mut Self {
        self.ops |= StrOps::PASCAL_CASE;
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
    pub fn into_string(self) -> String {
        todo!()
    }
}

impl<'a> Deref for VStr<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.raw
    }
}
