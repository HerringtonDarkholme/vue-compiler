pub struct TransfromContext {
}

pub trait Transformer {
    type IRNode;
    /// transform will change ir node inplace
    /// usually transform will have multiple passes
    fn transform(&self, node: &mut Self::IRNode);
}
