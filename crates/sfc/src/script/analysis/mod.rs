mod collect_import;

use crate::script::parse_script::{TsNode, TsPattern, TypeScript};
use crate::script::setup_context::SetupScriptContext;

pub use collect_import::{collect_setup_assets, collect_normal_import};
