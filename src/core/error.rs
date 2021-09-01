use super::SourceLocation;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum CompilationErrorKind {
    AbruptClosingOfEmptyComment,
    CDataInHtmlContent,
    DuplicateAttribute, // TODO
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInScriptHtmlCommentLikeText, // TODO
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingEndTagName,
    MissingWhitespaceBetweenAttributes,
    NestedComment,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedNullCharacter, // TODO
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,

    // Vue-specific parse errors
    InvalidEndTag,
    MissingEndTag,
    MissingInterpolationEnd,
    MissingDynamicDirectiveArgumentEnd, // TODO

    // TODO
    // transform errors
    VIfNoExpression,
    VIfSameKey,
    VElseNoAdjacentIf,
    VForNoExpression,
    VForMalformedExpression,
    VForTemplateKeyPlacement,
    VBindNoExpression,
    VOnNoExpression,
    VSlotUnexpectedDirectiveOnSlotOutlet,
    VSlotMixedSlotUsage,
    VSlotDuplicateSlotNames,
    VSlotExtraneousDefaultSlotChildren,
    VSlotMisplaced,
    VModelNoExpression,
    VModelMalformedExpression,
    VModelOnScopeVariable,
    InvalidExpression,
    KeepAliveInvalidChildren,

    // generic errors
    PrefixIdNotSupported,
    ModuleModeNotSupported,
    CacheHandlerNotSupported,
    ScopeIdNotSupported,

    // Special value for higher-order compilers to pick up the last code
    // to avoid collision of error codes. This should always be kept as the last item.
    ExtendPoint,
}

#[derive(Debug)]
pub struct CompilationError {
    pub kind: CompilationErrorKind,
    pub additional_message: Option<&'static str>,
    pub location: SourceLocation,
}

impl CompilationError {
    pub fn new(kind: CompilationErrorKind) -> Self {
        Self {
            kind,
            additional_message: None,
            location: Default::default(),
        }
    }
    pub fn with_location(mut self, loc: SourceLocation) -> Self {
        self.location = loc;
        self
    }
    pub fn with_additional_message(mut self, msg: &'static str) -> Self {
        self.additional_message = Some(msg);
        self
    }

    fn msg(&self) -> &'static str {
        msg(&self.kind)
    }
}

#[cold]
#[inline(never)]
fn msg(kind: &CompilationErrorKind) -> &'static str {
    use CompilationErrorKind as k;
    match *kind {
        k::AbruptClosingOfEmptyComment => "Illegal comment.",
        k::CDataInHtmlContent => "CDATA section is allowed only in XML context.",
        k::DuplicateAttribute => "Duplicate attribute.",
        k::EndTagWithAttributes => "End tag cannot have attributes.",
        k::EndTagWithTrailingSolidus => r#"Illegal "/" in tags."#,
        k::EofBeforeTagName => "Unexpected EOF in tag.",
        k::EofInCdata => "Unexpected EOF in CDATA section.",
        k::EofInComment => "Unexpected EOF in comment.",
        k::EofInScriptHtmlCommentLikeText => "Unexpected EOF in script.",
        k::EofInTag => "Unexpected EOF in tag.",
        k::IncorrectlyClosedComment => "Incorrectly closed comment.",
        k::IncorrectlyOpenedComment => "Incorrectly opened comment.",
        k::InvalidFirstCharacterOfTagName => "Illegal tag name. Use '&lt;' to print '<'.",
        k::UnexpectedEqualsSignBeforeAttributeName => "Attribute name was expected before '='.",
        k::MissingAttributeValue => "Attribute value was expected.",
        k::MissingEndTagName => "End tag name was expected.",
        k::MissingWhitespaceBetweenAttributes => "Whitespace was expected.",
        k::NestedComment => "Unexpected '<!--' in comment.",
        k::UnexpectedCharacterInAttributeName =>
            "Attribute name cannot contain U+0022 (\"), U+0027 ('), and U+003C (<).",
        k::UnexpectedCharacterInUnquotedAttributeValue =>
            "Unquoted attribute value cannot contain U+0022 (\"), U+0027 (\'), U+003C (<), U+003D (=), and U+0060 (`).",
        k::UnexpectedQuestionMarkInsteadOfTagName => "'<?' is allowed only in XML context.",
        k::UnexpectedNullCharacter => "Unexpected null character.",
        k::UnexpectedSolidusInTag => "Illegal '/' in tags.",

        // Vue-specific parse errors
        k::InvalidEndTag => "Invalid end tag.",
        k::MissingEndTag => "Element is missing end tag.",
        k::MissingInterpolationEnd => "Interpolation end sign was not found.",
        k::MissingDynamicDirectiveArgumentEnd =>
            "End bracket for dynamic directive argument was not found. Note that dynamic directive argument cannot contain spaces.",

        // transform errors
        k::VIfNoExpression => "v-if/v-else-if is missing expression.",
        k::VIfSameKey => "v-if/else branches must use unique keys.",
        k::VElseNoAdjacentIf => "v-else/v-else-if has no adjacent v-if.",
        k::VForNoExpression => "v-for is missing expression.",
        k::VForMalformedExpression => "v-for has invalid expression.",
        k::VForTemplateKeyPlacement => "<template v-for> key should be placed on the <template> tag.",
        k::VBindNoExpression => "v-bind is missing expression.",
        k::VOnNoExpression => "v-on is missing expression.",
        k::VSlotUnexpectedDirectiveOnSlotOutlet => "Unexpected custom directive on <slot> outlet.",
        k::VSlotMixedSlotUsage =>
            "Mixed v-slot usage on both the component and nested <template>. When there are multiple named slots, all slots should use <template> syntax to avoid scope ambiguity.",
        k::VSlotDuplicateSlotNames => "Duplicate slot names found. ",
        k::VSlotExtraneousDefaultSlotChildren =>
            r#"Extraneous children found when component already has explicitly named "default slot. These children will be ignored."#,
        k::VSlotMisplaced => "v-slot can only be used on components or <template> tags.",
        k::VModelNoExpression => "v-model is missing expression.",
        k::VModelMalformedExpression => "v-model value must be a valid JavaScript member expression.",
        k::VModelOnScopeVariable =>
            "v-model cannot be used on v-for or v-slot scope variables because they are not writable.",
        k::InvalidExpression => "Error parsing JavaScript expression: ",
        k::KeepAliveInvalidChildren => "<KeepAlive> expects exactly one child component.",

        // generic errors
        k::PrefixIdNotSupported =>
            r#""prefixIdentifiers" option is not supported in this build of compiler."#,
        k::ModuleModeNotSupported => "ES module mode is not supported in this build of compiler.",
        k::CacheHandlerNotSupported =>
            r#""cacheHandlers" option is only supported when the "prefixIdentifiers" option is enabled."#,
        k::ScopeIdNotSupported => r#""scopeId" option is only supported in module mode."#,
        k::ExtendPoint => "",
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(additional) = self.additional_message {
            write!(f, "{}{}", self.msg(), additional)
        } else {
            write!(f, "{}", self.msg())
        }
    }
}

/// This trait handles error occured in the compilation.
/// NB: clone bound is needed since scan/parse/ir/code gen
/// all requires ownership of a error report.
/// Rc/RefCell is a good way to implement ErrorHandler if
/// collecting errors in compilation pass is desired.
pub trait ErrorHandler: Clone {
    // cannot use mut ref due to borrow semantics
    // use RefCell as implementation
    fn on_error(&self, _: CompilationError) {}
}
