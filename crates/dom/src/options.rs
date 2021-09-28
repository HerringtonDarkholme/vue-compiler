use compiler::tokenizer::{TextMode, TokenizeOption};

fn get_text_mode(s: &str) -> TextMode {
    match s {
        "style" | "script" | "iframe" | "noscript" => TextMode::RawText,
        "textarea" | "title" => TextMode::RcData,
        _ => TextMode::Data,
    }
}

pub fn tokenize_option() -> TokenizeOption {
    TokenizeOption {
        get_text_mode,
        delimiters: ("{{".to_string(), "}}".to_string()),
    }
}
