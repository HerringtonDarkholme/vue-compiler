use super::{
    parser::{Directive, Element},
    runtime_helper::RuntimeHelper,
};

pub fn get_core_component(tag: &str) -> Option<RuntimeHelper> {
    use RuntimeHelper as RH;
    Some(match tag {
        "Teleport" | "teleport" => RH::TELEPORT,
        "Suspense" | "suspense" => RH::SUSPENSE,
        "KeepAlive" | "keep-alive" => RH::KEEP_ALIVE,
        "BaseTransition" | "base-transition" => RH::BASE_TRANSITION,
        _ => return None,
    })
}

pub fn is_core_component(tag: &str) -> bool {
    get_core_component(tag).is_some()
}

pub const fn yes(_: &str) -> bool {
    true
}
pub const fn no(_: &str) -> bool {
    false
}
