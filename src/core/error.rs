use super::SourceLocation;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum CompilationErrorKind {
    AbruptClosingOfEmptyComment,
    CDataInHtmlContent,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInScriptHtmlCommentLikeText,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingEndTagName,
    MissingWhitespaceBetweenAttributes,
    NestedComment,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,

    // Vue-specific parse errors
    InvalidEndTag,
    MissingEndTag,
    MissingInterpolationEnd,
    MissingDynamicDirectiveArgumentEnd,

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
    ExtendPoint
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

    #[cold]
    #[inline(never)]
    fn msg(&self) -> &'static str {
        use CompilationErrorKind as kind;
        match self.kind {
            kind::AbruptClosingOfEmptyComment => "Illegal comment.",
            kind::MissingInterpolationEnd => "Interpolation end sign was not found.",
            _ => todo!(),
        }
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
