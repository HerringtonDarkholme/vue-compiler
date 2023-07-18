use super::CliInput;
use anyhow::Result;
use compiler::compiler::{BaseCompiler, TemplateCompiler};
use compiler::SFCInfo;
use compiler::BindingTypes;
use dom::get_dom_pass;
use serde_yaml::to_writer;
use sfc::{parse_sfc, compile_script, SfcScriptCompileOptions, rewrite_default};
use std::io;

pub(super) fn compile_to_stdout(debug: CliInput) -> Result<()> {
    let (source, option, show) = debug;
    let sfc = parse_sfc(&source, Default::default());
    let script = compile_script(&sfc.descriptor, SfcScriptCompileOptions::new("anonymous"));
    let sfc_info = SFCInfo {
        inline: false,
        slotted: false,
        scope_id: None,
        binding_metadata: script.and_then(|s| s.bindings).unwrap_or_default(),
        self_name: "anonymous.vue".into(),
    };
    let dest = io::stdout;
    let compiler = BaseCompiler::new(dest, get_dom_pass, option);

    let template = if let Some(temp) = sfc.descriptor.template {
        temp.block.source
    } else {
        &source
    };
    let script = sfc
        .descriptor
        .scripts
        .first()
        .map(|s| s.block.source)
        .unwrap_or("");

    let tokens = compiler.scan(template);
    if show.dump_scan {
        let tokens: Vec<_> = compiler.scan(template).collect();
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

    compiler.transform(&mut ir, &sfc_info);
    if show.dump_transform {
        println!(r#"======= Transformed ========="#);
        let stdout = io::stdout();
        to_writer(stdout.lock(), &ir)?;
        println!(r#"======== End of Transform ========"#);
    }
    print_intro(&sfc_info);
    println!("{}", rewrite_default(script.into(), "__sfc__"));
    compiler.generate(ir, &sfc_info)?;
    print_outro(&sfc_info);
    Ok(())
}

fn print_intro(sfc: &SFCInfo) {
    println!("/* Analyzed bindings: {{");
    for (key, tpe) in sfc.binding_metadata.iter() {
        let tpe = match tpe {
            BindingTypes::Data => "data",
            BindingTypes::Props => "props",
            BindingTypes::Options => "options",
            BindingTypes::SetupMaybeRef => "setup-maybe-ref",
            _ => "setup",
        };
        println!("  \"{key}\": \"{tpe}\"");
    }
    println!("}} */");
}

fn print_outro(sfc: &SFCInfo) {
    println!("\n__sfc__.render = render");
    println!("__sfc__.__file = '{}'", sfc.self_name);
    println!("export default __sfc__");
}
