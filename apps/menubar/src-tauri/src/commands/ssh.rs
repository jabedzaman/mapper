use crate::ssh_config::{self, HostCheck, SshHost};

#[tauri::command]
pub fn list_ssh_hosts() -> Vec<SshHost> {
    ssh_config::list_hosts()
}

#[tauri::command]
pub fn check_ssh_host(alias: String) -> HostCheck {
    ssh_config::check_host(&alias)
}
