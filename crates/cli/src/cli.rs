use super::CliInput;
use anyhow::Result;
use compiler::{
    codegen::{self, CodeGenerator},
    converter::{self, Converter},
    parser, scanner,
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
use serde_yaml::to_writer;
use std::io;

pub(super) fn compile_to_stdout(debug: CliInput) -> Result<()> {
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

    let scanner = scanner::Scanner::new(option.scanning);
    let tokens = scanner.scan(&source, eh());
    if show.dump_scan {
        let tokens: Vec<_> = scanner.scan(&source, eh()).collect();
        println!(r#"============== Tokens ============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &tokens)?;
        println!(r#"========== End of Tokens =========="#);
    }

    let parser = parser::Parser::new(option.parsing);
    let ast = parser.parse(tokens, eh());
    if show.dump_parse {
        println!(r#"=============== AST =============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ast)?;
        println!(r#"=========== End of AST ==========="#);
    }

    let converter = converter::BaseConverter {
        err_handle: Box::new(eh()),
        option: option.conversion,
    };
    let mut ir = converter.convert_ir(ast);
    if show.dump_convert {
        println!(r#"============= IR ============"#);
        to_writer(io::stdout(), &ir)?;
        println!(r#"========== End of IR ==========="#);
    }

    let mut transformer = transformer::BaseTransformer::new(MergedPass::new(pass));
    transformer.transform(&mut ir);
    if show.dump_transform {
        println!(r#"======= Transformed ========="#);
        to_writer(io::stdout(), &ir)?;
        println!(r#"======== End of Transform ========"#);
    }

    let mut generator = codegen::CodeWriter::new(io::stdout(), option.codegen);
    generator.generate(ir)?;
    Ok(())
}
