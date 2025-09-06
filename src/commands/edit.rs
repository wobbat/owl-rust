use crate::internal::files;

/// Run the edit command to open files in editor
pub fn run(typ: &str, arg: &str) -> Result<(), String> {
    if arg.is_empty() {
        return Err("edit command requires a non-empty argument".to_string());
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
        _ => Err(format!("invalid edit type '{}'. Must be '{}' or '{}'",
            typ, crate::internal::constants::EDIT_TYPE_DOTS, crate::internal::constants::EDIT_TYPE_CONFIG)),
    }
}
