mod bench_util;

use std::rc::Rc;
use compiler::compiler::BaseCompiler;
use compiler::{
    chain,
    error::{NoopErrorHandler, RcErrHandle},
    compiler::{CompileOption, TemplateCompiler},
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

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn base_compile(source: &str, eh: RcErrHandle) {
    let shared: &mut [&mut dyn CorePassExt<_, _>] = &mut [
        &mut SlotFlagMarker,
        &mut ExpressionProcessor {
            option: &Default::default(),
            sfc_info: &Default::default(),
            err_handle: eh,
        },
    ];
    let pass = chain![
        TextOptimizer,
        EntityCollector::default(),
        PatchFlagMarker,
        SharedInfoPasses {
            passes: MergedPass::new(shared),
            shared_info: Scope::default(),
        },
    ];
    let mut s = Vec::new();
    let mut compiler = BaseCompiler::new(
        &mut s,
        pass,
        CompileOption {
            is_native_tag: |t| t != "draggable-header-view" && t != "tree-item",
            is_dev: false,
            ..Default::default()
        },
    );
    compiler.compile(source).unwrap();
}

fn test_enum_eq(c: &mut Criterion) {
    let eh = Rc::new(NoopErrorHandler);
    for (name, content) in bench_util::get_fixtures() {
        c.bench_with_input(BenchmarkId::new("compile", name), &content, |b, c| {
            b.iter(|| base_compile(c, eh.clone()));
        });
    }
}

criterion_group!(benches, test_enum_eq);
criterion_main!(benches);
