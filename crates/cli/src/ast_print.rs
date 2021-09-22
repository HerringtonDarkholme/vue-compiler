#![warn(dead_code, unused_variables)]
use compiler::{
    converter::{AstNode, AstRoot, Directive, Element},
    parser::{DirectiveArg, ElemProp, SourceNode, TextNode},
    tokenizer::{Attribute, AttributeValue},
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
            AstNode::Interpolation(source_node) => {
                format!(
                    "{}Interpolation{}",
                    "  ".repeat(level),
                    source_node.ast_string(level)
                )
            }
            AstNode::Comment(source_node) => {
                format!(
                    "{}Comment{}",
                    "  ".repeat(level),
                    source_node.ast_string(level)
                )
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
        let tag_name_string = format!("{}tag_name `{}`", "  ".repeat(level + 1), tag_name);
        let properties_string = properties
            .iter()
            .map(|prop| prop.ast_string(level + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let children_string = children
            .iter()
            .map(|node| node.ast_string(level + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let mut ret = format!("{}{}", "  ".repeat(level), element_string,);
        let next_level_string = vec![tag_name_string, properties_string, children_string]
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
            "{}Text {}..{} `{}`",
            "  ".repeat(level),
            start.offset,
            end.offset,
            self.text[0].raw
        )
    }
}

impl<'a> AstString for SourceNode<'a> {
    fn ast_string(&self, _: usize) -> String {
        let SourceLocation { start, end } = &self.location;
        // don't have prefix indent because the source code could be interpolation or comment
        format!(" {}..{} `{}`", start.offset, end.offset, self.source)
    }
}

impl<'a> AstString for ElemProp<'a> {
    fn ast_string(&self, level: usize) -> String {
        match self {
            ElemProp::Attr(attr) => attr.ast_string(level),
            ElemProp::Dir(dir) => {
                let Directive {
                    name,
                    argument,
                    modifiers,
                    expression,
                    location: SourceLocation { start, end },
                    ..
                } = dir;
                let name_string = format!("{}name `{}`", "  ".repeat(level + 1), name);
                let mut ret = format!(
                    "{}directive {}..{}\n{}",
                    "  ".repeat(level),
                    start.offset,
                    end.offset,
                    name_string,
                );
                if let Some(args) = argument {
                    ret += &format!("\n{}", args.ast_string(level + 1));
                }
                if !modifiers.is_empty() {
                    ret += &format!("\n{}modifiers `{}`", "  ".repeat(level + 1), modifiers.join(".") );
                }
                let expression = if let Some(value) = expression {
                    let AttributeValue {
                        content,
                        location: SourceLocation { start, end },
                    } = value;
                    format!(
                        "{}value {}..{} `{}`",
                        "  ".repeat(level + 1),
                        start.offset,
                        end.offset,
                        content.raw
                    )
                } else {
                    "".to_string()
                };
                if !expression.is_empty() {
                    ret += &format!("\n{}", expression);
                }
                ret
            }
        }
    }
}

impl<'a> AstString for Attribute<'a> {
    fn ast_string(&self, level: usize) -> String {
        let Attribute {
            name,
            value,
            name_loc:
                SourceLocation {
                    start: name_start,
                    end: name_end,
                },
            location: SourceLocation { start, end },
        } = self;
        let name_string = format!(
            "{}name {}..{} `{}`",
            "  ".repeat(level + 1),
            name_start.offset,
            name_end.offset,
            name
        );
        let value_string = if let Some(value) = value {
            let AttributeValue {
                content,
                location: SourceLocation { start, end },
            } = value;
            format!(
                "{}value {}..{} `{}`",
                "  ".repeat(level + 1),
                start.offset,
                end.offset,
                content.raw
            )
        } else {
            "".to_string()
        };
        let mut ret = format!(
            "{}attribute {}..{}\n{}",
            "  ".repeat(level),
            start.offset,
            end.offset,
            name_string,
        );
        if !value_string.is_empty() {
            ret += &format!("\n{}", value_string);
        }
        dbg!(&ret);
        ret
    }
}

impl<'a> AstString for DirectiveArg<'a> {
    fn ast_string(&self, level: usize) -> String {
        let arg_string = match self {
            DirectiveArg::Static(s) => {
                format!("{}argument `{}`", "  ".repeat(level), s)
            }
            DirectiveArg::Dynamic(d) => {
                format!("{}argument [{}]", "  ".repeat(level), d)
            }
        };
        arg_string
    }
}
