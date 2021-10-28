pub fn rewrite_default(input: String, _as_var: &'static str) -> String {
    if !has_any_default_export(&input) {
        return input;
    }
    todo!()
}

/*
const defaultExportRE = /((?:^|\n|;)\s*)export(\s*)default/
const namedDefaultExportRE = /((?:^|\n|;)\s*)export(.+)as(\s*)default/s
const exportDefaultClassRE =
  /((?:^|\n|;)\s*)export\s+default\s+class\s+([\w$]+)/
 * */

fn has_any_default_export(input: &str) -> bool {
    has_export_default(input) || has_named_default_export(input)
}
fn has_export_default(input: &str) -> bool {
    let _idx = input.find("export ");
    todo!()
}
fn has_named_default_export(_input: &str) -> bool {
    todo!()
}

fn is_start_of_statement(input: &str, pos: usize) -> bool {
    let input = input[..pos].trim_end_matches(char::is_whitespace);
    input.chars().last().unwrap_or(';') == ';'
}
