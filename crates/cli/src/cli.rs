use super::CliInput;
use anyhow::Result;
use compiler::{
    codegen::{self, CodeGenerator},
    converter::{self, Converter},
    parser, tokenizer,
    transformer::{
        self,
        collect_entities::EntityCollector,
        mark_patch_flag::PatchFlagMarker,
        mark_slot_flag::SlotFlagMarker,
        optimize_text::TextOptimizer,
        pass::{Scope, SharedInfoPasses},
        process_expression::ExpressionProcessor,
        CorePass, CorePassExt, MergedPass, Transformer,
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
    let error_handler = option.error_handler;
    let eh = || error_handler.clone();

    let tokenizer = tokenizer::Tokenizer::new(option.tokenization);
    let tokens = tokenizer.scan(&source, eh());

    let parser = parser::Parser::new(option.parsing);
    let ast = parser.parse(tokens, eh());

    let converter = converter::BaseConverter {
        err_handle: Box::new(eh()),
        option: option.conversion,
    };
    let mut ir = converter.convert_ir(ast);

    let mut transformer = transformer::BaseTransformer::new(MergedPass::new(pass));
    transformer.transform(&mut ir);

    let mut generator = codegen::CodeWriter::new(io::stdout(), option.codegen);
    generator.generate(ir)?;
    Ok(())
}
