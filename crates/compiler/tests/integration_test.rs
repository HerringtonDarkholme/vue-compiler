mod codegen_test;
mod common;
mod converter_test;
mod error_test;
mod parser_test;
mod scanner_test;
mod transformer_test;

#[macro_export]
macro_rules! assert_yaml {
    ($case: expr, $func: expr) => {
        let name = insta::_macro_support::AutoName;
        let val = $func($case);
        insta::assert_snapshot!(name, val, $case);
    };
}

// https://github.com/rust-lang/rust/issues/35853#issuecomment-816020221
#[macro_export]
macro_rules! meta_macro {
    ($func: ident) => {
        macro_rules! $func {
            // cannot use repetition in nested macro
            ($cases: expr) => {
                for case in $cases {
                    $crate::assert_yaml!(case, $func);
                }
            };
        }
    };
}
