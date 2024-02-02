use json::JsonValue;
use std::{
    collections::HashMap,
    env, fs,
    io::{BufRead, BufReader},
    process::{Child, ChildStdout, Command, Stdio},
};
use sway::{get_apps, get_workspaces, Node};

const SWAYMSG_BIN: &str = "/usr/bin/swaymsg";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("You must provide a path to an icons json file.");
    }

    let icons: HashMap<String, char> = get_icons(&args[1]);

    // should we consider to also subscribe to event "workspace"?
    // also should we test on event change (new, close, focus, title, fullscreen_mode, move, floating, urgent, mark)
    // and only rename workspaces on some event? If yes which events should we consider?
    let mut cmd: Child = Command::new(SWAYMSG_BIN)
        .args(["-rmt", "subscribe", "[\"window\",\"workspace\"]"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout: &mut ChildStdout = cmd.stdout.as_mut().unwrap();

    BufReader::new(stdout)
        .lines()
        .for_each(|_l: Result<String, std::io::Error>| {
            get_workspaces().members().for_each(|w: &JsonValue| {
                let apps: String = get_apps(Node::new(w));
                if apps.is_empty() {
                    clear_workspace_name(w["num"].to_string())
                } else {
                    set_workspace_name(w["num"].to_string(), &apps, &icons)
                }
            });
        });
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

// fn set_workspace_name(num: u8, apps: String, icons: JsonValue) {
fn set_workspace_name(num: String, apps: &str, icons: &HashMap<String, char>) {
    Command::new(SWAYMSG_BIN)
        .args([
            "rename",
            "workspace",
            "number",
            &num,
            "to",
            // format the workspace name
            &format!(
                "{}:{}",
                num,
                apps.lines()
                    .map(|l: &str| {
                        let ls: &str = l.split_once(' ').unwrap_or((l, "")).0;
                        if icons.contains_key(ls) {
                            // format!(" {}", &icons[ls])
                            " ".to_string() + &icons[ls].to_string()
                        } else {
                            "  \u{f22d}".to_string()
                        }
                    })
                    .collect::<String>()
            ),
        ])
        .output()
        .expect("Failed to rename workspace");
}

fn clear_workspace_name(num: String) {
    Command::new(SWAYMSG_BIN)
        .args(["rename", "workspace", "number", &num, "to", &num])
        .output()
        .expect("Failed to rename workspace");
}
