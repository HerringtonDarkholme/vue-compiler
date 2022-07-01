// sadly current html decode crate requires std::io::Write not fmt
use std::fmt::{self, Write};
use super::named_chars::NAMED_CHAR_REF;
use std::sync::LazyLock;

static MAX_CR_NAME_LEN: LazyLock<usize> =
    LazyLock::new(|| NAMED_CHAR_REF.keys().copied().map(str::len).max().unwrap());

type DecodeResult<'a> = Result<&'a str, fmt::Error>;
pub fn decode_entities<W: Write>(s: &str, mut w: W, as_attr: bool) -> fmt::Result {
    let mut src = s;
    while let Some(idx) = src.find('&') {
        let (decoded, next) = src.split_at(idx);
        w.write_str(decoded)?;
        src = next;
        if src.starts_with("&#") {
            src = decode_numeric_ref(src, &mut w)?;
        } else {
            src = decode_named_ref(src, &mut w, as_attr)?;
        }
    }
    w.write_str(src)
}

fn decode_named_ref<W: Write>(s: &str, mut w: W, as_attr: bool) -> DecodeResult {
    debug_assert!(s.starts_with('&'));
    let mut src = &s[1..];
    if !src.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        w.write_char('&')?;
        return Ok(src);
    }
    let max_len = MAX_CR_NAME_LEN.min(src.len());
    let entry = (2..=max_len)
        .rev()
        .map(|i| &src[..i])
        .find_map(|k| NAMED_CHAR_REF.get_entry(k));
    let (key, val) = match entry {
        Some(entry) => entry,
        None => {
            w.write_char('&')?;
            return Ok(src);
        }
    };
    let semi = key.ends_with(';');
    src = &src[key.len()..];
    if as_attr && !semi && src.starts_with(|c: char| c == '=' || c.is_ascii_alphanumeric()) {
        w.write_char('&')?;
        w.write_str(key)?;
        Ok(src)
    } else {
        w.write_str(val)?;
        Ok(src)
    }
}
fn decode_numeric_ref<W: Write>(s: &str, mut w: W) -> DecodeResult {
    debug_assert!(s.starts_with("&#"));
    let (num, next) = if let Some(src) = s.strip_prefix("&#x") {
        // hex
        let cnt = src.chars().take_while(|c| c.is_ascii_hexdigit()).count();
        match u32::from_str_radix(&src[..cnt], 16) {
            Ok(n) => {
                if src[cnt..].starts_with(';') {
                    (n, &src[cnt + 1..])
                } else {
                    (n, &src[cnt..])
                }
            }
            Err(_) => return Ok(src),
        }
    } else {
        // num
        let src = &s[2..];
        let cnt = src.chars().take_while(|c| c.is_numeric()).count();
        match src[..cnt].parse() {
            Ok(n) => {
                if src[cnt..].starts_with(';') {
                    (n, &src[cnt + 1..])
                } else {
                    (n, &src[cnt..])
                }
            }
            Err(_) => return Ok(src),
        }
    };
    let num = match num {
        0 => 0xfffd,
        n if n > 0x10ffff => 0xfffd,
        0xd800..=0xdfff => 0xfffd,
        0xfdd0..=0xfdef => num,           // noop
        n if (n & 0xfffe) == 0xfffe => n, // noop
        0x80..=0x9f => CCR_REPLACEMENTS[num as usize - 0x80],
        num => num,
    };
    if let Some(c) = char::from_u32(num) {
        w.write_char(c)?;
        Ok(next)
    } else {
        Ok(next)
    }
}

// https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state
const CCR_REPLACEMENTS: &[u32] = &[
    0x20ac, // 0x80
    0x81,   // 0x81, noop
    0x201a, // 0x82
    0x0192, // 0x83
    0x201e, // 0x84
    0x2026, // 0x85
    0x2020, // 0x86
    0x2021, // 0x87
    0x02c6, // 0x88
    0x2030, // 0x89
    0x0160, // 0x8a
    0x2039, // 0x8b
    0x0152, // 0x8c
    0x8d,   // 0x8d, noop
    0x017d, // 0x8e
    0x8f,   // 0x8f, noop
    0x90,   // 0x90, noop
    0x2018, // 0x91
    0x2019, // 0x92
    0x201c, // 0x93
    0x201d, // 0x94
    0x2022, // 0x95
    0x2013, // 0x96
    0x2014, // 0x97
    0x02dc, // 0x98
    0x2122, // 0x99
    0x0161, // 0x9a
    0x203a, // 0x9b
    0x0153, // 0x9c
    0x9d,   // 0x9d, noop
    0x017e, // 0x9e
    0x0178, // 0x9f
];

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_html_decode() {
        let data = [
            ("", ""),
            ("Håll älgen, Örjan!", "Håll älgen, Örjan!"),
            ("&lt;p&gt;hej!&lt;/p&gt;", "<p>hej!</p>"),
            ("hej&#x3B;&#x20;hå", "hej; hå"),
            ("&quot;width&#x3A;&#32;3px&#59;&quot;", "\"width: 3px;\""),
            ("&#x2b;", "+"),
        ];
        for &(input, expected) in data.iter() {
            let mut actual = String::new();
            decode_entities(input, &mut actual, false).unwrap();
            assert_eq!(&actual, expected);
        }
    }
}
