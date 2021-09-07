use super::{AstNode, CoreConvertInfo, IRNode, IfBranch, IfNodeIR};
pub fn convert_if(nodes: Vec<AstNode>, key: usize) -> IRNode<CoreConvertInfo> {
    let mut branches = nodes
        .into_iter()
        .map(|n| convert_if_branch(n, key))
        .collect();
    IRNode::If(IfNodeIR { branches, info: () })
}

fn convert_if_branch(node: AstNode, start_key: usize) -> IfBranch<CoreConvertInfo> {
    IfBranch {
        children: vec![],
        condition: todo!(),
    }
}

#[cfg(test)]
mod test {
    fn test() {
        let cases = vec![
            r#"
<p v-if="false">a</p>
<p v-else v-if="true">b</p>
<p v-else>c</p>"#,
        ];
    }
}
