use anyhow::{bail, Context, Result};
use expanduser;
use jwalk::WalkDir;
use std::{collections::HashMap, env};

mod skimmer;
mod tmux;

const TMUX_STATUS_LEFT: usize = 25;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 0 {
        bail!("You must supply a directory!")
    }
    let walk_path = expanduser::expanduser(&args[0])?
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
        let prefix_stripped = entry.strip_prefix(&walk_path).unwrap();
        let suffix_stripped = entry.strip_suffix("/.git").unwrap().to_owned();
        let parts = prefix_stripped.split("/").collect::<Vec<_>>();
        let dir_key = parts[1].replace(".", "_").to_owned();
        let sub_key = parts[2..parts.len() - 1].join("/");
        //let mut sub_key = parts[parts.len() - 2].to_owne();
        skim_strings.push(format!("{}:{}", &dir_key, &sub_key).to_string());
        entry_map
            .entry(dir_key)
            .or_default()
            .insert(sub_key, suffix_stripped);
    }
    skim_strings.append(&mut tmux::get_sessions());
    let entries = skim_strings.join("\n");
    let results = skimmer::get_choice(entries);
    // If we results is None, don't do anything
    let query = if let Some(q) = results {
        q
    } else {
        return Ok(());
    };
    let choice_parts = query[0].split(":").collect::<Vec<_>>();
    let session_type = choice_parts[0].to_owned();
    if choice_parts.len() == 1 {
        // This was a query with no matches, so create a new session
        return tmux::create_session(&session_type, "~".into());
    } else if session_type == "session" {
        return tmux::switch_client(choice_parts[1]);
    } else {
        let session_name = get_final_session_name(&session_type, choice_parts[1]);
        let type_map = entry_map
            .get(choice_parts[0])
            .expect(&format!("Can't find {} in the type map!", session_type));
        let session_path = type_map
            .get(choice_parts[1])
            .with_context(|| format!("Can't find '{}' in the type map!", choice_parts[1]))?;
        return tmux::create_session(&session_name, session_path.into());
    }
}

fn get_final_session_name(session_type: &str, session_name: &str) -> String {
    // Ideally we want the name to be 'session_type:session_name'
    // If that's too long try and shorten session_name, then type,
    // then drop the type completely.
    // If the session name is > TMUX_STATUS_LEFT - just use it alone
    if session_name.len() >= TMUX_STATUS_LEFT {
        // First tico it and check that length
        let shortened = tico::tico(session_name, None);
        if shortened.len() <= TMUX_STATUS_LEFT {
            return shortened;
        } else {
            // If the tico length is still too long, get the last part of the path
            let last_part = session_name
                .split('/')
                .into_iter()
                .next_back()
                .unwrap_or(session_name);
            if last_part.len() <= TMUX_STATUS_LEFT {
                return last_part.into();
            } else {
                return last_part[(last_part.len() - TMUX_STATUS_LEFT + 4)..last_part.len()].into();
            }
        }
    }
    // Now we'll try and build 'session_type|session_name'
    if session_type.len() + session_name.len() + 1 <= TMUX_STATUS_LEFT {
        return format!("{}|{}", session_type, session_name);
    }
    // Now try with the full session_name plus abbreviated type
    if session_name.len() + 3 <= TMUX_STATUS_LEFT {
        return format!("{}|{}", &session_type[0..2], session_name);
    }
    // Now do it all again with the shortened name
    let shortened = tico::tico(session_name, None);
    if session_type.len() + shortened.len() + 1 <= TMUX_STATUS_LEFT {
        return format!("{}|{}", session_type, shortened);
    }
    if shortened.len() + 3 <= TMUX_STATUS_LEFT {
        return format!("{}|{}", &session_type[0..2], shortened);
    }
    // If we got all this way, just use the last TMUX_STATUS_LEFT - 4 chars
    session_name[(session_name.len() - TMUX_STATUS_LEFT + 4)..session_name.len()].into()
}
