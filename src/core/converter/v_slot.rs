use super::{
    super::error::{CompilationError, CompilationErrorKind as ErrorKind},
    BaseConverter as BC, BaseIR, CoreConverter, Element,
};
use crate::core::{parser::ElementType, util::find_dir};

pub fn convert_v_slot<'a>(bc: &BC, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

pub fn check_build_as_slot(bc: &BC, e: &Element) -> bool {
    if let Some(found) = find_dir(e, "slot") {
        debug_assert!(e.tag_type != ElementType::Template);
        let dir = found.get_ref();
        if !e.is_component() {
            let error = CompilationError::new(ErrorKind::VSlotMisplaced)
                .with_location(dir.location.clone());
            bc.emit_error(error);
        }
    }
    todo!()
}
