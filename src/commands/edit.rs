use crate::internal::files;
use anyhow::{Result, anyhow};

/// Run the edit command to open files in editor
pub fn run(typ: &str, arg: &str) -> Result<()> {
    if arg.is_empty() {
        return Err(anyhow!("edit command requires a non-empty argument"));
    }

    match typ {
        crate::internal::constants::EDIT_TYPE_DOTS => {
            let path = files::get_dotfile_path(arg)?;
            files::open_editor(&path)
        }
        crate::internal::constants::EDIT_TYPE_CONFIG => {
            let path = files::find_config_file(arg)?;
            files::open_editor(&path)
        }
        _ => Err(anyhow!(
            "invalid edit type '{}'. Must be '{}' or '{}'",
            typ,
            crate::internal::constants::EDIT_TYPE_DOTS,
            crate::internal::constants::EDIT_TYPE_CONFIG
        )),
    }
}
