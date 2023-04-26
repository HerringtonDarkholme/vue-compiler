use super::{TsNode, TsPattern, TypeScript};
use crate::script::{apply_ref_transform};

pub fn process_normal_script(ast: TsNode) {
    rewrite_export_or_walk_declaration(ast);
    // apply reactivity transform
    apply_ref_transform();
    move_script_before_setup();
}

fn move_script_before_setup() {
    // TODO: do we really need this?
}

// <script> after <script setup>
// we need to move the block up so that `const __default__` is
// declared before being used in the actual component definition
pub fn process_setup_script(ast: TsNode) {}

fn rewrite_export_or_walk_declaration(ast: TsNode) {}
