use std::collections::{HashSet, HashMap};
use compiler::{BindingMetadata, BindingTypes};
use super::parse_script::TsNode;
use crate::SfcDescriptor;
use super::SfcScriptCompileOptions;

struct ImportBinding<'a> {
    is_type: bool,
    imported: &'a str,
    source: &'a str,
    local: &'a str,
    is_from_setup: bool,
    is_used_in_template: bool,
}

enum PropsDeclType {
    // define variants for PropsDeclType enum
}

enum EmitsDeclType {
    // define variants for EmitsDeclType enum
}

struct PropTypeData {
    // define fields for PropTypeData struct
}

#[derive(Default)]
struct SetupBindings<'a> {
    binding_metadata: BindingMetadata<'a>,
    helper_imports: HashSet<String>,
    user_imports: HashMap<String, ImportBinding<'a>>,
    script_bindings: HashMap<String, BindingTypes>,
    setup_bindings: HashMap<String, BindingTypes>,
}

#[derive(Default)]
struct ExportRelated<'a> {
    default_export: Option<TsNode<'a>>,
    has_default_export_name: bool,
    has_default_export_render: bool,
}

type ObjectExpression = ();

#[derive(Default)]
struct Misc {
    has_define_expose_call: bool,
    has_await: bool,
    has_inlined_ssr_render_fn: bool,
    declared_types: HashMap<String, Vec<String>>,
}

#[derive(Default)]
struct PropRelated<'a> {
    has_define_props_call: bool,
    type_declared_props: HashMap<String, PropTypeData>,
    props_runtime_decl: Option<TsNode<'a>>,
    props_runtime_defaults: Option<ObjectExpression>,
    props_destructure_decl: Option<TsNode<'a>>,
    props_destructure_rest_id: Option<String>,
    props_type_decl: Option<PropsDeclType>,
    props_type_decl_raw: Option<TsNode<'a>>,
    props_identifier: Option<String>,
    props_destructured_bindings: HashMap<String, HashMap<String, bool>>,
}

#[derive(Default)]
struct EmitRelated<'a> {
    has_define_emit_call: bool,
    emits_runtime_decl: Option<TsNode<'a>>,
    emits_type_decl: Option<EmitsDeclType>,
    emits_type_decl_raw: Option<TsNode<'a>>,
    emit_identifier: Option<String>,
    type_declared_emits: HashSet<String>,
}

#[derive(Default)]
struct SetupScriptData<'a> {
    bindings: SetupBindings<'a>,
    props: PropRelated<'a>,
    emits: EmitRelated<'a>,
    exports: ExportRelated<'a>,
}

pub struct SetupScriptContext<'a, 'b> {
    data: SetupScriptData<'a>,
    sfc: &'b SfcDescriptor<'a>,
    options: &'b SfcScriptCompileOptions<'a>,
    is_ts: bool,
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
            data: SetupScriptData::default(),
            sfc,
            options,
            is_ts,
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
        self.data
            .bindings
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
}

fn is_import_used(_local: &str) -> bool {
    todo!()
}
