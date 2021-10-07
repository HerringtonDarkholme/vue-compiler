/// I am sorry but json.rs requires io::Write
/// but I can only present fmt::Write
use std::fmt::{Result as Ret, Write};

const QU: char = '"';
const BS: char = '\\';
const BB: char = 'b';
const TT: char = 't';
const NN: char = 'n';
const FF: char = 'f';
const RR: char = 'r';
const UU: char = 'u';
const __: char = '_';

// Look up table for characters that need escaping in a product string
static ESCAPED: [char; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[inline(never)]
fn write_string_complex<W: Write>(mut w: W, string: &str, mut start: usize) -> Ret {
    w.write_str(&string[..start])?;

    for (index, ch) in string.bytes().enumerate().skip(start) {
        let escape = ESCAPED[ch as usize];
        if escape != __ {
            w.write_str(&string[start..index])?;
            w.write_char('\\')?;
            w.write_char(escape as char)?;
            start = index + 1;
        }
        if escape == 'u' {
            write!(w, "{:04x}", ch)?;
        }
    }
    w.write_str(&string[start..])?;

    w.write_char('"')
}

#[inline(always)]
pub fn write_json_string<W: Write>(string: &str, mut w: W) -> Ret {
    w.write_char('"')?;

    for (index, ch) in string.bytes().enumerate() {
        if ESCAPED[ch as usize] != __ {
            return write_string_complex(w, string, index);
        }
    }

    w.write_str(string)?;
    w.write_char('"')
}

#[cfg(test)]
mod test {
    use super::*;

    fn stringify(s: &str) -> String {
        let mut ret = String::new();
        write_json_string(s, &mut ret).unwrap();
        ret
    }

    #[test]
    fn stringify_str() {
        assert_eq!(stringify("Foo"), "\"Foo\"");
    }

    #[test]
    fn stringify_escaped_characters() {
        assert_eq!(
            stringify("\r____\n___\t\u{8}\u{c}\\\"__"),
            r#""\r____\n___\t\b\f\\\"__""#
        );
    }

    #[test]
    fn stringify_dont_escape_forward_slash() {
        assert_eq!(stringify("foo/bar"), r#""foo/bar""#);
    }

    #[test]
    fn stringify_escaped() {
        assert_eq!(
            stringify("http://www.google.com/\t"),
            r#""http://www.google.com/\t""#
        );
    }

    #[test]
    fn stringify_control_escaped() {
        assert_eq!(
            stringify("foo\u{1f}bar\u{0}baz"),
            r#""foo\u001fbar\u0000baz""#
        );
    }
}
