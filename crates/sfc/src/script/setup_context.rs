use std::collections::{HashSet, HashMap};
use compiler::BindingMetadata;
use super::parse_script::TsNode;
use crate::SfcDescriptor;
use super::SfcScriptCompileOptions;

pub struct ImportBinding<'a> {
    pub is_type: bool,
    pub imported: &'a str,
    pub source: &'a str,
    pub local: &'a str,
    pub is_from_setup: bool,
    pub is_used_in_template: bool,
}

struct TypeScope;

#[derive(Default)]
pub struct Analysis<'a> {
    // import / type analysis
    scope: Option<TypeScope>,
    global_scopes: Option<Vec<TypeScope>>,
    pub user_imports: HashMap<String, ImportBinding<'a>>,
    // codegen
    binding_metadata: BindingMetadata<'a>,
    helper_imports: HashSet<String>,
}

pub enum Issue {
    Error(String),
    Warning(String),
}

#[derive(Default)]
struct Macros<'a> {
    // macros presence check
    has_define_props_call: bool,
    has_define_emit_call: bool,
    has_define_expose_call: bool,
    has_default_export_name: bool,
    has_default_export_render: bool,
    has_define_options_call: bool,
    has_define_model_call: bool,
    has_define_slots_call: bool,
    // defineProps
    props_identifier: Option<String>,
    props_runtime_decl: Option<TsNode<'a>>,
    props_type_decl: Option<TsNode<'a>>,
    props_destructure_decl: Option<TsNode<'a>>,
    props_destructured_bindings: HashMap<String, TsNode<'a>>,
    props_destructure_rest_id: Option<String>,
    props_runtime_defaults: Option<TsNode<'a>>,
    // defineEmits
    emits_runtime_decl: Option<TsNode<'a>>,
    emits_type_decl: Option<TsNode<'a>>,
    emit_identifier: Option<String>,
    // defineModel
    model_decls: HashMap<String, String>,
    // defineOptions
    options_runtime_decl: Option<TsNode<'a>>,
}

pub struct SetupScriptContext<'a, 'b> {
    pub analysis: Analysis<'a>,
    macros: Macros<'a>,
    sfc: &'b SfcDescriptor<'a>,
    options: &'b SfcScriptCompileOptions<'a>,
    is_ts: bool,
    issues: Vec<Issue>,
}

impl<'a, 'b> SetupScriptContext<'a, 'b> {
    pub fn new(sfc: &'b SfcDescriptor<'a>, options: &'b SfcScriptCompileOptions<'a>) -> Self {
        let lang = sfc.scripts[0].get_lang();
        let is_ts = lang == "ts" || lang == "tsx";
        debug_assert! {
            // either a single script or two scripts have the same lang
            sfc.scripts.len() == 1 || sfc.scripts[1].get_lang() == lang
        };
        Self {
            analysis: Analysis::default(),
            macros: Macros::default(),
            sfc,
            options,
            is_ts,
            issues: vec![],
        }
    }
    fn need_check_template(&self) -> bool {
        // template usage check is only needed in non-inline mode
        // so we can skip the work if inlineTemplate is true.
        if self.options.inline_template || !self.is_ts {
            return false;
        }
        let Some(template) = &self.sfc.template else {
            return false;
        };
        let attrs = &template.block.attrs;
        // only check if the template is inside SFC and is written in html
        !attrs.contains_key("src")
            && attrs.get("lang").cloned().flatten().unwrap_or("html") == "html"
    }

    pub fn get_registered_import(&self, local: &str) -> Option<&ImportBinding> {
        self.analysis.user_imports.get(local)
    }

    pub fn register_script_import(
        &mut self,
        source: TsNode,
        local: TsNode,
        imported: Option<TsNode>,
        is_type: bool,
    ) {
        let source = self.script_text(&source);
        let local = self.script_text(&local);
        let imported = imported.map(|n| self.script_text(&n)).unwrap_or("default");
        self.register_user_import(
            source, local, imported, is_type, /*is_from_setup*/ false,
        )
    }

    pub fn register_setup_import(
        &mut self,
        source: TsNode,
        local: TsNode,
        imported: Option<TsNode>,
        is_type: bool,
    ) {
        let source = self.setup_text(&source);
        let local = self.setup_text(&local);
        let imported = imported.map(|n| self.setup_text(&n)).unwrap_or("default");
        self.register_user_import(
            source, local, imported, is_type, /*is_from_setup*/ true,
        )
    }

    fn register_user_import(
        &mut self,
        source: &'a str,
        local: &'a str,
        imported: &'a str,
        is_type: bool,
        is_from_setup: bool,
    ) {
        let is_used_in_template = self.need_check_template() && is_import_used(local);
        let user_import = ImportBinding {
            is_type,
            imported, // named or default
            local,
            source,
            is_from_setup,
            is_used_in_template,
        };
        self.analysis
            .user_imports
            .insert(local.to_string(), user_import);
    }

    pub fn script_text(&self, node: &TsNode) -> &'a str {
        let range = node.range();
        let script = self.sfc.scripts.iter().find(|s| !s.is_setup()).unwrap();
        &script.block.source[range]
    }

    pub fn setup_text(&self, node: &TsNode) -> &'a str {
        let range = node.range();
        let script = self.sfc.scripts.iter().find(|s| s.is_setup()).unwrap();
        &script.block.source[range]
    }

    pub fn warn(&mut self, warning: String) {
        self.issues.push(Issue::Warning(warning))
    }

    pub fn error(&mut self, error: String) {
        self.issues.push(Issue::Error(error))
    }
}

fn is_import_used(_local: &str) -> bool {
    todo!()
}
