use std::env;
use winreg::RegKey;
use winreg::enums::*;

pub fn register_context_menu() -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r#"Software\Classes\Directory\shell\RobocopyGUI"#;

    let (key, _disp) = hkcu.create_subkey(path)?;
    key.set_value("", &"Copy with Robocopy GUI")?;
    key.set_value("Icon", &"imageres.dll,-5302")?; // arbitrary icon

    let command_path = format!(r#"{}\command"#, path);
    let (cmd_key, _) = hkcu.create_subkey(command_path)?;

    let exe_path = env::current_exe()?;
    let exe_str = exe_path.to_str().unwrap_or("");
    let cmd_string = format!("\"{}\" \"%1\"", exe_str);

    cmd_key.set_value("", &cmd_string)?;

    Ok(())
}

pub fn remove_context_menu() -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r#"Software\Classes\Directory\shell\RobocopyGUI"#;
    // recursively delete
    let _ = hkcu.delete_subkey_all(path);
    Ok(())
}
