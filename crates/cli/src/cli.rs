use super::CliInput;
use anyhow::Result;
use compiler::{
    compiler::{BaseCompiler, TemplateCompiler},
    transformer::{
        collect_entities::EntityCollector,
        mark_patch_flag::PatchFlagMarker,
        mark_slot_flag::SlotFlagMarker,
        optimize_text::TextOptimizer,
        pass::{Scope, SharedInfoPasses},
        process_expression::ExpressionProcessor,
        CorePass, CorePassExt, MergedPass,
    },
};
use std::io;

pub(super) fn compile_to_stdout<'a>(debug: CliInput) -> Result<()> {
    let (source, option, show) = debug;
    let shared: &mut [&mut dyn CorePassExt<_, _>] = &mut [
        &mut SlotFlagMarker,
        &mut ExpressionProcessor {
            option: &Default::default(),
        },
    ];
    let pass: &mut [&mut dyn CorePass<_>] = &mut [
        &mut TextOptimizer,
        &mut EntityCollector::default(),
        &mut PatchFlagMarker,
        &mut SharedInfoPasses {
            passes: MergedPass::new(shared),
            shared_info: Scope::default(),
        },
    ];
    let mut compiler = BaseCompiler::new(io::stdout(), pass, option);
    compiler.compile(&source)?;
    Ok(())
}
