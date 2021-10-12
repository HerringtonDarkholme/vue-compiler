mod stringify_static;
mod warn_dom_usage;

use warn_dom_usage::UsageWarner;
use compiler::transformer::{
    CorePass,
    collect_entities::EntityCollector,
    mark_patch_flag::PatchFlagMarker,
    mark_slot_flag::SlotFlagMarker,
    optimize_text::TextOptimizer,
    pass::{Scope, SharedInfoPasses},
    process_expression::ExpressionProcessor,
    normalize_props::NormalizeProp,
    hoist_static::HoistStatic,
};
use compiler::converter::BaseConvertInfo;
use compiler::{SFCInfo, chain};
use compiler::compiler::CompileOption;
use std::marker::PhantomData;

fn get_dom_pass<'a>(
    sfc_info: &'a SFCInfo<'a>,
    opt: &CompileOption,
) -> impl CorePass<BaseConvertInfo<'a>> {
    let prefix_identifier = opt.transforming().prefix_identifier;
    let shared = chain![
        SlotFlagMarker,
        HoistStatic::new(opt.cache_handlers),
        ExpressionProcessor {
            prefix_identifier,
            sfc_info,
            err_handle: opt.error_handler.clone(),
        },
    ];
    chain![
        PatchFlagMarker,
        UsageWarner(opt.error_handler.clone()),
        TextOptimizer,
        EntityCollector::default(),
        NormalizeProp,
        SharedInfoPasses {
            passes: shared,
            shared_info: Scope::default(),
            pd: PhantomData,
        },
    ]
}
