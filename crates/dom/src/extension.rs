use compiler::error::ErrorKind;

pub mod dom_helper {
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
    VHtmlNoExpression,
    VHtmlWithChildren,
    VTextNoExpression,
    VTextWithChildren,
    VModelOnInvalidElement,
    VModelArgOnElement,
    VModelOnFileInputElement,
    VModelUnnecessaryValue,
    VShowNoExpression,
    TransitionInvalidChildren,
    IgnoredSideEffectTag,
}

impl ErrorKind for DomError {
    fn msg(&self) -> &'static str {
        use DomError::*;
        match self {
          VHtmlNoExpression => "v-html is missing expression.",
          VHtmlWithChildren => "v-html will override element children.",
          VTextNoExpression => "v-text is missing expression.",
          VTextWithChildren => "v-text will override element children.",
          VModelOnInvalidElement => "v-model can only be used on <input>, <textarea> and <select> elements.",
          VModelArgOnElement => "v-model argument is not supported on plain elements.",
          VModelOnFileInputElement => "v-model cannot be used on file inputs since they are read-only. Use a v-on:change listener instead.",
          VModelUnnecessaryValue => "Unnecessary value binding used alongside v-model. It will interfere with v-model's behavior.",
          VShowNoExpression => "v-show is missing expression.",
          TransitionInvalidChildren => "<Transition> expects exactly one child element or component.",
          IgnoredSideEffectTag => "Tags with side effect (<script> and <style>) are ignored in client component templates."
        }
    }
}
