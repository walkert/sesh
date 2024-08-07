use std::path::PathBuf;

use tmux_interface::{ListSessions, NewSession, SwitchClient, Tmux};

//pub fn get_sessions() -> Vec<String> {
pub fn get_sessions() -> Vec<String> {
    let mut sessions: Vec<String> = Vec::new();
    let output = Tmux::with_command(ListSessions::new()).output().unwrap();
    let stdout = String::from_utf8(output.stdout()).expect("unable to get sessions");
    for session in stdout.split("\n").into_iter() {
        let parts = session.split(":").collect::<Vec<_>>();
        if parts.len() == 1 {
            continue;
        }
        sessions.push(format!("session:{}", parts[0].to_owned()));
    }
    sessions
}

pub fn switch_client(name: &str) {
    let _ = Tmux::with_command(SwitchClient::new().target_session(name)).status();
}

pub fn create_session(name: &str, start_path: PathBuf) {
    //println!("Creating session {} with path {:?}", name, start_path);
    Tmux::new()
        .add_command(
            NewSession::new()
                .detached()
                .session_name(name)
                .start_directory(start_path.to_string_lossy()),
        )
        .output()
        .unwrap();
    let _ = Tmux::with_command(SwitchClient::new().target_session(name)).status();
}
