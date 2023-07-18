mod collect_import;
mod process_script;

use crate::script::parse_script::{TsNode, TsPattern, TypeScript};
use crate::script::setup_context::SetupScriptContext;

pub use collect_import::{collect_setup_assets, collect_normal_import};
pub use process_script::{process_setup_script, process_normal_script};
