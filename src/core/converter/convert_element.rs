use super::{BaseConverter, BaseIR, Element};

pub fn convert_element<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    // 1. resolve dynamic component
    // 1.5 v-is (deprecated)
    // 2. built-in components (Teleport, Transition, KeepAlive, Suspense...)
    // 3. user component (from setup bindings)
    // 4. Self referencing component (inferred from filename)
    // 5. user component (resolve)
    todo!()
}
pub fn convert_component<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}
pub fn convert_template<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

pub fn resolve_setup_reference() {
    todo!()
}
