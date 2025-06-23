use i3ipc::{event::Event, I3Connection, I3EventListener, Subscription};
use std::{collections::HashMap, env, fs};
use sway::{get_apps, get_tree, Node};

fn main() {
    let icons_path = env::args().nth(1)
        .expect("Usage: workspaces <icons.json>");
    if let Err(e) = listen(&icons_path) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn listen(icons_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let icons = load_icons(icons_path)?;
    let mut conn = I3Connection::connect()?;
    let mut listener = I3EventListener::connect()?;
    let subs = [Subscription::Workspace, Subscription::Window];
    listener.subscribe(&subs)?;

    for event in listener.listen() {
        match event? {
            Event::WindowEvent(_) | Event::WorkspaceEvent(_) => {
                update_workspaces(&mut conn, &icons)
            }
            _ => (),
        }
    }
    Ok(())
}

fn load_icons(path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let parsed = json::parse(&data)?;
    Ok(parsed.entries()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

fn format_workspace_apps(apps: &str, icons: &HashMap<String, String>) -> String {
    apps.lines()
        .map(|line| {
            let app = line.split_once(' ').map(|(a, _)| a).unwrap_or(line);
            icons.get(app)
                .map(|icon| format!(" {}", icon))
                .unwrap_or_else(|| "  \u{f22d}".to_string())
        })
        .collect()
}

fn rename_workspace(conn: &mut I3Connection, num: &str, name: &str) {
    let cmd = format!("rename workspace number {} to '{}'", num, name);
    if let Err(e) = conn.run_command(&cmd) {
        eprintln!("Failed to rename workspace {num}: {e}");
    }
}

fn update_workspaces(conn: &mut I3Connection, icons: &HashMap<String, String>) {
    for output in get_tree()["nodes"].members() {
        for ws in output["nodes"].members() {
            let num = ws["num"].to_string();
            let apps = get_apps(Node::new(ws));
            if apps.is_empty() {
                rename_workspace(conn, &num, &num);
            } else {
                let name = format!("{}:{}", num, format_workspace_apps(&apps, icons));
                rename_workspace(conn, &num, &name);
            }
        }
    }
}