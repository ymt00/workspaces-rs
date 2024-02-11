use i3ipc::event::Event;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use json::JsonValue;
use std::{collections::HashMap, env, fs};
use sway::{get_apps, get_workspaces, Node};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("You must provide a path to an icons json file.");
    }

    listen(args[1].as_str());
}

fn listen(icons_path: &str) {
    let icons: HashMap<String, char> = get_icons(icons_path);
    let mut connection: I3Connection = I3Connection::connect().expect("Failed to connect");
    let mut listener: I3EventListener = I3EventListener::connect().expect("Failed to connect");
    let subs: [Subscription; 2] = [Subscription::Workspace, Subscription::Window];

    listener.subscribe(&subs).expect("Failed to subscribe");
    for event in listener.listen() {
        match event.unwrap() {
            Event::WindowEvent(_w) => set_workspaces_name(&mut connection, &icons),
            Event::WorkspaceEvent(_w) => set_workspaces_name(&mut connection, &icons),
            _ => (),
        }
    }
}

fn get_icons(icons_path: &str) -> HashMap<String, char> {
    json::parse(
        fs::read_to_string(icons_path)
            .expect("Unable to read icons file")
            .as_str(),
    )
    .unwrap()
    .entries()
    .map(|i: (&str, &JsonValue)| (i.0.to_string(), i.1.to_string().chars().next().unwrap()))
    .collect()
}

fn format_workspace_name(apps: &str, icons: &HashMap<String, char>) -> String {
    apps.lines()
        .map(|l: &str| {
            // we split by ' ' and default to the line
            let ls: &str = l.split_once(' ').unwrap_or((l, "")).0;
            if icons.contains_key(ls) {
                " ".to_string() + icons[ls].to_string().as_str()
            } else {
                "  \u{f22d}".to_string()
            }
        })
        .collect::<String>()
}

fn clear_workspace_name(conn: &mut I3Connection, num: String) {
    let run_command =
        conn.run_command(format!("rename workspace number {} to '{}'", num, num).as_str());
    let _ = &run_command.expect("Failed to rename workspace");
}

fn set_workspace_name(conn: &mut I3Connection, num: String, apps: String) {
    let _ = &conn
        .run_command(format!("rename workspace number {} to '{}:{}'", num, num, apps).as_str())
        .expect("Failed to rename workspace");
}

fn set_workspaces_name(conn: &mut I3Connection, icons: &HashMap<String, char>) {
    get_workspaces().members().for_each(|w: &JsonValue| {
        let apps: String = get_apps(Node::new(w));
        if apps.is_empty() {
            clear_workspace_name(conn, w["num"].to_string())
        } else {
            set_workspace_name(
                conn,
                w["num"].to_string(),
                format_workspace_name(&apps, icons),
            )
        }
    });
}
