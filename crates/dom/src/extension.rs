use compiler::error::ErrorKind;

pub mod DomHelper {
    use compiler::flags::RuntimeHelper as RH;
    pub const V_MODEL_RADIO: RH = RH(RH::INTERNAL_MAX);
    pub const V_MODEL_CHECKBOX: RH = RH(RH::INTERNAL_MAX + 1);
    pub const V_MODEL_TEXT: RH = RH(RH::INTERNAL_MAX + 2);
    pub const V_MODEL_SELECT: RH = RH(RH::INTERNAL_MAX + 3);
    pub const V_MODEL_DYNAMIC: RH = RH(RH::INTERNAL_MAX + 4);
    pub const V_ON_WITH_MODIFIERS: RH = RH(RH::INTERNAL_MAX + 5);
    pub const V_ON_WITH_KEYS: RH = RH(RH::INTERNAL_MAX + 6);
    pub const V_SHOW: RH = RH(RH::INTERNAL_MAX + 7);
    pub const TRANSITION: RH = RH(RH::INTERNAL_MAX + 8);
    pub const TRANSITION_GROUP: RH = RH(RH::INTERNAL_MAX + 9);

    pub const DOM_HELPER_MAP: &[&str] = &[
        "vModelRadio",
        "vModelCheckbox",
        "vModelText",
        "vModelSelect",
        "vModelDynamic",
        "withModifiers",
        "withKeys",
        "vShow",
        "Transition",
        "TransitionGroup",
    ];
}

pub enum DomError {
    HtmlNoExpression,
    HtmlWithChildren,
    TextNoExpression,
    TextWithChildren,
    ModelOnInvalidElement,
    ModelArgOnElement,
    ModelOnFileInputElement,
    ModelUnnecessaryValue,
    ShowNoExpression,
    TransitionInvalidChildren,
    IgnoredSideEffectTag,
}

impl ErrorKind for DomError {
    fn msg(&self) -> &'static str {
        use DomError::*;
        match self {
          HtmlNoExpression => "v-html is missing expression.",
          HtmlWithChildren => "v-html will override element children.",
          TextNoExpression => "v-text is missing expression.",
          TextWithChildren => "v-text will override element children.",
          ModelOnInvalidElement => "v-model can only be used on <input>, <textarea> and <select> elements.",
          ModelArgOnElement => "v-model argument is not supported on plain elements.",
          ModelOnFileInputElement => "v-model cannot be used on file inputs since they are read-only. Use a v-on:change listener instead.",
          ModelUnnecessaryValue => "Unnecessary value binding used alongside v-model. It will interfere with v-model's behavior.",
          ShowNoExpression => "v-show is missing expression.",
          TransitionInvalidChildren => "<Transition> expects exactly one child element or component.",
          IgnoredSideEffectTag => "Tags with side effect (<script> and <style>) are ignored in client component templates."
        }
    }
}
