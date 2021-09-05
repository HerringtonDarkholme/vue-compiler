/*
 * Transform
 * hoistStatic
 * transformExpression
 * merge_text
 * vOnce
 * vMemo
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
pub fn transform_text() {}
pub fn transform_expression() {}
pub fn transform_memo() {}

enum NodeChange<T: 'static> {
    Replace(Vec<T>),
    Delete,
}

trait TransformOp {}
