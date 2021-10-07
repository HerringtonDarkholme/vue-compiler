use super::CliInput;
use anyhow::Result;
use compiler::{
    chain,
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
        Transformer,
    },
};
use serde_yaml::to_writer;
use std::{io, marker::PhantomData};

pub(super) fn compile_to_stdout(debug: CliInput) -> Result<()> {
    let (source, option, show) = debug;
    let prefix_identifier = option.transforming().prefix_identifier;
    let shared = chain![
        SlotFlagMarker,
        ExpressionProcessor {
            prefix_identifier,
            sfc_info: &Default::default(),
            err_handle: option.error_handler.clone(),
        },
    ];
    let pass = chain![
        TextOptimizer,
        EntityCollector::default(),
        PatchFlagMarker,
        SharedInfoPasses {
            passes: shared,
            shared_info: Scope::default(),
            pd: PhantomData,
        },
    ];
    let eh = || option.error_handler.clone();

    let scanner = scanner::Scanner::new(option.scanning());
    let tokens = scanner.scan(&source, eh());
    if show.dump_scan {
        let tokens: Vec<_> = scanner.scan(&source, eh()).collect();
        println!(r#"============== Tokens ============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &tokens)?;
        println!(r#"========== End of Tokens =========="#);
    }

    let parser = parser::Parser::new(option.parsing());
    let ast = parser.parse(tokens, eh());
    if show.dump_parse {
        println!(r#"=============== AST =============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ast)?;
        println!(r#"=========== End of AST ==========="#);
    }

    let converter = converter::BaseConverter {
        err_handle: eh(),
        sfc_info: Default::default(),
        option: option.converting(),
    };
    let mut ir = converter.convert_ir(ast);
    if show.dump_convert {
        println!(r#"============= IR ============"#);
        to_writer(io::stdout(), &ir)?;
        println!(r#"========== End of IR ==========="#);
    }

    let mut transformer = transformer::BaseTransformer::new(pass);
    transformer.transform(&mut ir);
    if show.dump_transform {
        println!(r#"======= Transformed ========="#);
        to_writer(io::stdout(), &ir)?;
        println!(r#"======== End of Transform ========"#);
    }

    let mut generator =
        codegen::CodeWriter::new(io::stdout(), option.codegen(), Default::default());
    generator.generate(ir)?;
    Ok(())
}
