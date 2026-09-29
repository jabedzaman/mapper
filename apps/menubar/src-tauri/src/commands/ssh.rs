use crate::ssh_config::{self, ConfigDoc, HostCheck, SshHost};

#[tauri::command]
pub fn list_ssh_hosts() -> Vec<SshHost> {
    ssh_config::list_hosts()
}

#[tauri::command]
pub fn check_ssh_host(alias: String) -> HostCheck {
    ssh_config::check_host(&alias)
}

#[tauri::command]
pub fn get_ssh_config() -> Result<String, String> {
    ssh_config::read_raw()
}

#[tauri::command]
pub fn save_ssh_config(contents: String) -> Result<(), String> {
    ssh_config::write_raw(&contents)
}

#[tauri::command]
pub fn get_ssh_config_doc() -> Result<ConfigDoc, String> {
    ssh_config::parse_doc()
}

#[tauri::command]
pub fn save_ssh_config_doc(doc: ConfigDoc) -> Result<(), String> {
    ssh_config::write_raw(&ssh_config::serialize_doc(&doc))
}
