use super::SourceLocation;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(PartialEq, Eq, Debug)]
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
    MissingDynamicDirectiveArgumentEnd,
    UnexpectedContentAfterDynamicDirective,
    MissingDirectiveName,
    MissingDirectiveArg,
    MissingDirectiveMod,
    InvalidVSlotModifier,

    // transform errors
    VIfNoExpression,
    VIfSameKey,
    VIfDuplicateDir,
    VElseNoAdjacentIf,
    VForNoExpression,
    VForMalformedExpression,
    VForTemplateKeyPlacement, // TODO
    VBindNoExpression,
    VOnNoExpression,
    VSlotUnexpectedDirectiveOnSlotOutlet,
    VSlotMixedSlotUsage,
    VSlotTemplateMisplaced,
    VSlotDuplicateSlotNames,
    VSlotExtraneousDefaultSlotChildren,
    VSlotMisplaced,
    // TODO
    VModelNoExpression,
    VModelMalformedExpression,
    VModelOnScopeVariable,
    InvalidExpression,

    UnexpectedDirExpression,
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

pub struct CompilationError {
    pub kind: CompilationErrorKind,
    pub additional_message: Option<String>,
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
    pub fn with_additional_message(mut self, msg: String) -> Self {
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
    use CompilationErrorKind::*;
    match *kind {
        AbruptClosingOfEmptyComment => "Illegal comment.",
        CDataInHtmlContent => "CDATA section is allowed only in XML context.",
        DuplicateAttribute => "Duplicate attribute.",
        EndTagWithAttributes => "End tag cannot have attributes.",
        EndTagWithTrailingSolidus => r#"Illegal "/" in tags."#,
        EofBeforeTagName => "Unexpected EOF in tag.",
        EofInCdata => "Unexpected EOF in CDATA section.",
        EofInComment => "Unexpected EOF in comment.",
        EofInScriptHtmlCommentLikeText => "Unexpected EOF in script.",
        EofInTag => "Unexpected EOF in tag.",
        IncorrectlyClosedComment => "Incorrectly closed comment.",
        IncorrectlyOpenedComment => "Incorrectly opened comment.",
        InvalidFirstCharacterOfTagName => "Illegal tag name. Use '&lt;' to print '<'.",
        UnexpectedEqualsSignBeforeAttributeName => "Attribute name was expected before '='.",
        MissingAttributeValue => "Attribute value was expected.",
        MissingEndTagName => "End tag name was expected.",
        MissingWhitespaceBetweenAttributes => "Whitespace was expected.",
        NestedComment => "Unexpected '<!--' in comment.",
        UnexpectedCharacterInAttributeName =>
         "Attribute name cannot contain U+0022 (\"), U+0027 ('), and U+003C (<).",
        UnexpectedCharacterInUnquotedAttributeValue =>
            "Unquoted attribute value cannot contain U+0022 (\"), U+0027 (\'), U+003C (<), U+003D (=), and U+0060 (`).",
        UnexpectedQuestionMarkInsteadOfTagName => "'<?' is allowed only in XML context.",
        UnexpectedNullCharacter => "Unexpected null character.",
        UnexpectedSolidusInTag => "Illegal '/' in tags.",

        // Vue-specific parse errors
        InvalidEndTag => "Invalid end tag.",
        MissingEndTag => "Element is missing end tag.",
        MissingInterpolationEnd => "Interpolation end sign was not found.",
        MissingDynamicDirectiveArgumentEnd =>
            "End bracket for dynamic directive argument was not found. Note that dynamic directive argument cannot contain spaces.",
        UnexpectedContentAfterDynamicDirective =>
            "Unexpected content was found after a closed dynamic argument. Add a dot as separator if it is a modifier.",
        MissingDirectiveName => "Legal directive name was expected.",
        MissingDirectiveArg => "Directive argument was expected.",
        MissingDirectiveMod => "Directive modifier was expected.",
        InvalidVSlotModifier => "v-slot does not take modifier.",

        // transform errors
        VIfNoExpression => "v-if/v-else-if is missing expression.",
        VIfSameKey => "v-if/else branches must use unique keys.",
        VIfDuplicateDir => "Duplicate v-if/else-if/else. Use v-else-if instead.",
        VElseNoAdjacentIf => "v-else/v-else-if has no adjacent v-if.",
        VForNoExpression => "v-for is missing expression.",
        VForMalformedExpression => "v-for has invalid expression.",
        VForTemplateKeyPlacement => "<template v-for> key should be placed on the <template> tag.",
        VBindNoExpression => "v-bind is missing expression.",
        VOnNoExpression => "v-on is missing expression.",
        VSlotUnexpectedDirectiveOnSlotOutlet => "Unexpected custom directive on <slot> outlet.",
        VSlotMixedSlotUsage =>
            "Mixed v-slot usage on both the component and nested <template>. When there are multiple named slots, all slots should use <template> syntax to avoid scope ambiguity.",
        VSlotDuplicateSlotNames => "Duplicate slot names found. ",
        VSlotExtraneousDefaultSlotChildren =>
            r#"Extraneous children found when component already has explicitly named "default slot. These children will be ignored."#,
        VSlotMisplaced => "v-slot can only be used on components or <template> tags.",
        VSlotTemplateMisplaced => "<template v-slot> can only be used as a component's direct child.",
        VModelNoExpression => "v-model is missing expression.",
        VModelMalformedExpression => "v-model value must be a valid JavaScript member expression.",
        VModelOnScopeVariable =>
            "v-model cannot be used on v-for or v-slot scope variables because they are not writable.",
        InvalidExpression => "Error parsing JavaScript expression: ",
        UnexpectedDirExpression => "This directive does not accept any epxression.",
        KeepAliveInvalidChildren => "<KeepAlive> expects exactly one child component.",

        // generic errors
        PrefixIdNotSupported =>
            r#""prefixIdentifiers" option is not supported in this build of compiler."#,
        ModuleModeNotSupported => "ES module mode is not supported in this build of compiler.",
        CacheHandlerNotSupported =>
            r#""cacheHandlers" option is only supported when the "prefixIdentifiers" option is enabled."#,
        ScopeIdNotSupported => r#""scopeId" option is only supported in module mode."#,
        ExtendPoint => "",
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(additional) = &self.additional_message {
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
pub trait ErrorHandler {
    // cannot use mut ref due to borrow semantics
    // use RefCell as implementation
    fn on_error(&self, _: CompilationError) {}
}

#[derive(Clone)]
pub struct VecErrorHandler {
    errors: Rc<RefCell<Vec<CompilationError>>>,
}
impl Default for VecErrorHandler {
    fn default() -> Self {
        Self {
            errors: Rc::new(RefCell::new(vec![])),
        }
    }
}

impl ErrorHandler for VecErrorHandler {
    fn on_error(&self, e: CompilationError) {
        self.errors.borrow_mut().push(e);
    }
}

#[cfg(test)]
pub mod test {
    use super::ErrorHandler;
    #[derive(Clone)]
    pub struct TestErrorHandler;
    impl ErrorHandler for TestErrorHandler {}
}
