use crate::core::PreambleHelper;
use bitflags::bitflags;

bitflags! {
  pub struct RuntimeHelper: u64 {
    const FRAGMENT                  = 1 << 0;
    const TELEPORT                  = 1 << 1;
    const SUSPENSE                  = 1 << 2;
    const KEEP_ALIVE                = 1 << 3;
    const BASE_TRANSITION           = 1 << 4;
    const OPEN_BLOCK                = 1 << 5;
    const CREATE_BLOCK              = 1 << 6;
    const CREATE_ELEMENT_BLOCK      = 1 << 7;
    const CREATE_VNODE              = 1 << 8;
    const CREATE_ELEMENT_VNODE      = 1 << 9;
    const CREATE_COMMENT            = 1 << 10;
    const CREATE_TEXT               = 1 << 11;
    const CREATE_STATIC             = 1 << 12;
    const RESOLVE_COMPONENT         = 1 << 13;
    const RESOLVE_DYNAMIC_COMPONENT = 1 << 14;
    const RESOLVE_DIRECTIVE         = 1 << 15;
    const RESOLVE_FILTER            = 1 << 16;
    const WITH_DIRECTIVES           = 1 << 17;
    const RENDER_LIST               = 1 << 18;
    const RENDER_SLOT               = 1 << 19;
    const CREATE_SLOTS              = 1 << 20;
    const TO_DISPLAY_STRING         = 1 << 21;
    const MERGE_PROPS               = 1 << 22;
    const NORMALIZE_CLASS           = 1 << 23;
    const NORMALIZE_STYLE           = 1 << 24;
    const NORMALIZE_PROPS           = 1 << 25;
    const GUARD_REACTIVE_PROPS      = 1 << 26;
    const TO_HANDLERS               = 1 << 27;
    const CAMELIZE                  = 1 << 28;
    const CAPITALIZE                = 1 << 29;
    const TO_HANDLER_KEY            = 1 << 30;
    const SET_BLOCK_TRACKING        = 1 << 31;
    const PUSH_SCOPE_ID             = 1 << 32;
    const POP_SCOPE_ID              = 1 << 33;
    const WITH_SCOPE_ID             = 1 << 34;
    const WITH_CTX                  = 1 << 35;
    const UNREF                     = 1 << 36;
    const IS_REF                    = 1 << 37;
    const WITH_MEMO                 = 1 << 38;
    const IS_MEMO_SAME              = 1 << 39;
  }
}

impl PreambleHelper<RuntimeHelper> for RuntimeHelper {
    fn collect_helper(&mut self, helper: RuntimeHelper) {
       *self |= helper;
    }
    fn generate_imports(&self) -> String {
        todo!()
    }
}
