use std::env;
use std::fs;
use serde_json::Value;
use std::process::{Child,Command,Stdio,ChildStdout,Output};
use std::io::{BufReader,BufRead,Lines};
use regex::Regex;

fn main() {
    let args: Vec<String>= env::args().collect();
    
    if args.len() < 2 {
        return;
    }
        
    let icons: Value = read_icon_file(args[1].to_string());

    subscribe_window_event(icons);
}

fn read_icon_file(icons_path: String) -> Value{
    let data: String = fs::read_to_string(icons_path).expect("Unable to read icons file");

    let icons: Value = serde_json::from_str(&data).unwrap();
    
    icons
}

fn subscribe_window_event(icons: Value) {
    let mut cmd: Child = Command::new("swaymsg")
        .args(["-rmt", "subscribe", "[\"window\"]"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout: &mut ChildStdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader: BufReader<&mut ChildStdout> = BufReader::new(stdout);
        let stdout_lines: Lines<BufReader<&mut ChildStdout>> = stdout_reader.lines();

        for _line in stdout_lines {            
            for w in get_workspaces().as_array().unwrap() {
                let mut apps: String = String::new();

                get_apps(w, &mut apps);

                set_workspace_name(w["num"].to_owned(), &mut apps, icons.clone());
            }
        }
    }

    cmd.wait().unwrap();
}

fn get_workspaces() -> Value {
    let output: Output = Command::new("swaymsg")
        .args(["-rt", "get_workspaces"])
        .output()
        .expect("Failed to execute command");

    let data = String::from_utf8_lossy(&output.stdout);
    
    serde_json::from_str(&data).unwrap()
}

fn get_apps (w: &Value, apps: &mut std::string::String) {
    let re = Regex::new(r"H|V|T|S|\[|\]").unwrap();
    let rep_before = json::parse(&w["representation"].to_string()).unwrap().to_string();
    let rep_after = re.replace_all(&rep_before, "");
    
    if !rep_after.is_empty() && rep_after != "null" {
        apps.push_str(&rep_after.replace(" ", "\n"))
    }
    get_nodes_apps(w["nodes"].to_owned(), apps);
    get_nodes_apps(w["floating_nodes"].to_owned(), apps);
}

fn get_nodes_apps(nodes: Value, apps: &mut std::string::String) {
    for n in nodes.as_array().unwrap() {
        let mut json_app_id = json::parse(&n["app_id"].to_string()).unwrap().to_string();
        if !json_app_id.is_empty() && n["app_id"].to_string() != "null"{
            if !apps.is_empty() {
                json_app_id.insert_str(0, "\n")
            }
            apps.push_str(&json_app_id)
        }

        get_nodes_apps(n["nodes"].to_owned(), apps);
        get_nodes_apps(n["floating_nodes"].to_owned(), apps);
    }
}

fn set_workspace_name(num: Value, apps: &mut std::string::String, icons: Value) {
    let mut apps_icon: String = String::new();

    for a in apps.lines() {
        apps_icon.push_str(&(" ".to_string()+&icons[a].to_string()));
    }

    let mut number = num.to_string();

    if !apps_icon.is_empty() {
        number = number.to_owned()+":"+&apps_icon;
    }

    Command::new("swaymsg")
        .args(["rename", "workspace", "number", &num.to_string(), "to", &number])
        .output().expect("Failed to rename workspace");

}