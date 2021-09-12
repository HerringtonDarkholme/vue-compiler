/*!
Transform IRNode.
This module contains the canonical transformations from vue-next and
the original ones for the parity of features not implemented in Convert.

## Canonical
* hoistStatic
* transformExpression
* vOnce
* vMemo
* trackScopes

## Original
* collectHelper: track all helpers used in AST. Vue track it by helper/helperString.
* mergeText: merge consecutive text call

 */
pub trait Transformer {
    type IR;
    /// transform will change ir node inplace
    /// usually transform will have multiple passes
    fn transform(&self, node: &mut Self::IR);
}

// default transforms
pub fn hoist_static() {}
pub fn track_v_for_slot_scopes() {}
pub fn track_slot_scopes() {}
pub fn merge_text_call() {}
pub fn prefix_expression() {}
pub fn transform_memo() {}
pub fn transform_once() {}
pub fn post_process_v_for_child() {
    // 1. inject key to slot
    // 2. Reuse the child's codegenNode but mark it as a block.
}

enum NodeChange<T: 'static> {
    Replace(Vec<T>),
    Delete,
}

trait TransformOp {}
