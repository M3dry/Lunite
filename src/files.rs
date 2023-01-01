use std::{io::Write, path::PathBuf};

use lunite::{Profiles, Task};

const DEFAULT_GLOBAL_TASKS: &str = r#"{"profiles":[{"name": "Default"}],"current":1}"#;
const DEFAULT_LOCAL_TASKS: &str = r#"[]"#;

pub fn get_profiles() -> (Profiles, PathBuf) {
    let path = xdg::BaseDirectories::with_prefix("lunite")
        .unwrap()
        .place_data_file("lunite.json")
        .unwrap();

    if !std::path::Path::new(&path).exists() {
        write!(
            std::fs::File::create(&path).unwrap(),
            "{}",
            DEFAULT_GLOBAL_TASKS
        )
        .unwrap();
    }

    (serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap(), path)
}

pub fn get_local_tasks(create: bool) -> std::io::Result<(Vec<Task>, PathBuf)> {
    let mut path = std::env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_lunite = std::fs::read_dir(p)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == std::ffi::OsString::from(".lunite.json"));
        if has_lunite {
            let mut path = PathBuf::from(p);
            path.push(".lunite.json");
            return Ok((
                serde_json::from_str(&std::fs::read_to_string(&path)?)?,
                path,
            ));
        }
    }
    if create {
        path.push(".lunite.json");
        write!(
            std::fs::File::create(&path).unwrap(),
            "{}",
            DEFAULT_LOCAL_TASKS
        )
        .unwrap();
        path.pop();
        return Ok((vec![], path));
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Ran out of places to find .lunite.json",
        ))
    }
}
