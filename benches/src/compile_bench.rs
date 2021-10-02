mod bench_util;

use compiler::compiler::BaseCompiler;
use compiler::{
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

fn base_compile(source: &str) {
    let shared: &mut [&mut dyn CorePassExt<_, _>] = &mut [
        &mut SlotFlagMarker,
        &mut ExpressionProcessor {
            option: &Default::default(),
            sfc_info: &Default::default(),
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
    let mut s = Vec::new();
    let mut compiler = BaseCompiler::new(
        &mut s,
        pass,
        CompileOption {
            is_native_tag: |t| t != "draggable-header-view" && t != "tree-item",
            ..Default::default()
        },
    );
    compiler.compile(source).unwrap();
}

fn test_enum_eq(c: &mut Criterion) {
    for (name, content) in bench_util::get_fixtures() {
        c.bench_with_input(BenchmarkId::new("compile", name), &content, |b, c| {
            b.iter(|| base_compile(c));
        });
    }
}

criterion_group!(benches, test_enum_eq);
criterion_main!(benches);
