use crate::{
    converter::{AstNode, AstRoot, Element},
    parser::{ElemProp, SourceNode, TextNode},
    SourceLocation,
};

pub trait AstString {
    fn ast_string(&self, level: usize) -> String;
}

impl<'a> AstString for AstRoot<'a> {
    fn ast_string(&self, level: usize) -> String {
        let SourceLocation { start, end } = &self.location;
        let root_string = format!("Root {}..{}", start.offset, end.offset);
        let children_string = self
            .children
            .iter()
            .map(|node| node.ast_string(level + 1))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}", root_string, children_string)
    }
}

impl<'a> AstString for AstNode<'a> {
    fn ast_string(&self, level: usize) -> String {
        match self {
            AstNode::Element(element) => element.ast_string(level),
            AstNode::Text(text) => text.ast_string(level),
            AstNode::Interpolation(source_node) | AstNode::Comment(source_node) => {
                source_node.ast_string(level)
            }
        }
    }
}

impl<'a> AstString for Element<'a> {
    fn ast_string(&self, level: usize) -> String {
        let SourceLocation { start, end } = &self.location;
        let element_string = format!("Element {}..{}", start.offset, end.offset,);
        let Element {
            tag_name,
            properties,
            children,
            ..
        } = self;
        let properties_string = properties
            .iter()
            .map(|prop| prop.ast_string(level + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let children_string = self
            .children
            .iter()
            .map(|node| node.ast_string(level + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let mut ret = format!("{}{}", "  ".repeat(level), element_string,);
        let next_level_string = vec![children_string]
            .into_iter()
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if !next_level_string.is_empty() {
            ret += &format!("\n{}", next_level_string);
        }
        ret
    }
}

impl<'a> AstString for TextNode<'a> {
    fn ast_string(&self, level: usize) -> String {
        let SourceLocation { start, end } = &self.location;
        format!(
            "{}Text {}..{}",
            "  ".repeat(level),
            start.offset,
            end.offset
        )
    }
}

impl<'a> AstString for SourceNode<'a> {
    fn ast_string(&self, level: usize) -> String {
        let SourceLocation { start, end } = &self.location;
        format!(
            "{}Interpolation {}..{}",
            "  ".repeat(level),
            start.offset,
            end.offset
        )
    }
}

impl<'a> AstString for ElemProp<'a> {
    fn ast_string(&self, level: usize) -> String {
        match self {
            ElemProp::Attr(attr) => {
                let name_string = format!("{}", "  ".repeat(level + 1));
                let value_string = format!("{}", "  ".repeat(level + 1));
                let mut ret = format!("{}\n{}", "  ".repeat(level), name_string);
                if !value_string.is_empty() {
                    ret += &format!("\n{}", value_string);
                }
                dbg!(&ret);
                ret
            }
            ElemProp::Dir(dir) => {
                unimplemented!() // TODO
            }
        }
    }
}
