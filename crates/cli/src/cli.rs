use super::CliInput;
use anyhow::Result;
use compiler::compiler::{get_base_passes, BaseCompiler, TemplateCompiler};
use serde_yaml::to_writer;
use std::io;

pub(super) fn compile_to_stdout(debug: CliInput) -> Result<()> {
    let (source, option, show) = debug;
    let sfc_info = Default::default();
    let passes = get_base_passes(&sfc_info, &option);
    let mut compiler = BaseCompiler::new(io::stdout(), passes, option);

    let tokens = compiler.scan(&source);
    if show.dump_scan {
        let tokens: Vec<_> = compiler.scan(&source).collect();
        println!(r#"============== Tokens ============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &tokens)?;
        println!(r#"========== End of Tokens =========="#);
    }

    let ast = compiler.parse(tokens);
    if show.dump_parse {
        println!(r#"=============== AST =============="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ast)?;
        println!(r#"=========== End of AST ==========="#);
    }

    let mut ir = compiler.convert(ast, &sfc_info);
    if show.dump_convert {
        println!(r#"============= IR ============"#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ir)?;
        println!(r#"========== End of IR ==========="#);
    }

    compiler.transform(&mut ir);
    if show.dump_transform {
        println!(r#"======= Transformed ========="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ir)?;
        println!(r#"======== End of Transform ========"#);
    }

    compiler.generate(ir, &sfc_info)?;
    Ok(())
}
