mod bench_util;

use compiler::compiler::BaseCompiler;
use compiler::{
    compiler::{CompileOption, TemplateCompiler},
    error::VecErrorHandler,
    parser::ParseOption,
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
use std::rc::Rc;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

fn base_compile(source: &str) {
    let shared: &mut [&mut dyn CorePassExt<_, _>] = &mut [
        &mut SlotFlagMarker,
        &mut ExpressionProcessor {
            option: &Default::default(),
            binding_metadata: &Default::default(),
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
    let parsing = ParseOption {
        is_native_element: |t| t != "draggable-header-view" && t != "tree-item",
        ..Default::default()
    };
    let mut compiler = BaseCompiler::new(
        &mut s,
        pass,
        CompileOption {
            scanning: Default::default(),
            parsing,
            conversion: Default::default(),
            transformation: Default::default(),
            codegen: Default::default(),
            error_handler: Rc::new(VecErrorHandler::default()),
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
