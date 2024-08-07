use expanduser;
use std::{collections::HashMap, env, process::exit};

use jwalk::WalkDir;

mod skimmer;
mod tmux;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let walk_path = expanduser::expanduser(&args[0])
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let walker = WalkDir::new(&walk_path)
        .skip_hidden(false)
        .max_depth(6)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir()
                    && entry.file_name().to_str().unwrap_or("").ends_with(".git")
                {
                    return Some(entry.path().to_string_lossy().into_owned());
                }
            }
            None
        });
    let mut entry_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut skim_strings: Vec<String> = Vec::new();
    for entry in walker.into_iter() {
        //println!("{}", &entry);
        let prefix_stripped = entry.strip_prefix(&walk_path).unwrap();
        let suffix_stripped = entry.strip_suffix("/.git").unwrap().to_owned();
        let parts = prefix_stripped.split("/").collect::<Vec<_>>();
        let dir_key = parts[1].to_owned();
        let sub_key = parts[2..parts.len() - 1].join("/");
        //let mut sub_key = parts[parts.len() - 2].to_owne();
        skim_strings.push(format!("{}:{}", &dir_key, &sub_key).to_string());
        entry_map
            .entry(dir_key)
            .or_default()
            .insert(sub_key, suffix_stripped);
    }
    //exit(0);
    skim_strings.append(&mut tmux::get_sessions());
    let entries = skim_strings.join("\n");
    let results = skimmer::get_choice(entries);
    let query = if let Some(q) = results {
        q
    } else {
        std::process::exit(1);
    };
    if args[args.len() - 1] == "dry" {
        println!("{:?}", query);
        exit(0);
    }
    let choice_parts = query[0].split(":").collect::<Vec<_>>();
    let session_type = choice_parts[0].to_owned();
    if choice_parts.len() == 1 {
        // This was a query with no matches, so create a new session
        tmux::create_session(&session_type, "~".into());
    } else if session_type == "session" {
        tmux::switch_client(choice_parts[1]);
    } else {
        let session_name = format!("{}|{}", session_type, choice_parts[1]);
        let type_map = entry_map
            .get(choice_parts[0])
            .expect(&format!("Can't find {} in the type map!", session_type));
        let session_path = type_map.get(choice_parts[1]).expect(&format!(
            "Can't find '{}' in the type map!",
            choice_parts[1]
        ));
        //let session_path = entry_map.entry(choice_parts[0].to_owned()).entry(choice_parts[1].to_owned())
        tmux::create_session(&session_name, session_path.into());
    }
}
