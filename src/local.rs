use std::path::PathBuf;

use lunite::{filter, Profile, Reset, Task};

use crate::files;

pub fn args(args: &[String]) {
    let (tasks, mut path) = files::get_local_tasks(true).unwrap();
    path.pop();
    let mut local = Local {
        profile: Profile {
            tasks,
            name: path.iter().last().unwrap().to_string_lossy().to_string(),
            links: vec![],
        },
        path: path.clone(),
    };
    let args_len = args.len();

    match args_len {
        0 => local.show(ShowMethod::All),
        _ if args[0] == "rm" => {
            let mut iter = args[1..].iter();
            let mut ids = vec![];
            let mut force = false;
            while let Some(id) = iter.next() {
                if let Ok(id) = id.parse::<usize>() {
                    ids.push(id)
                } else if id == "f" {
                    local.profile.remove_nth_task(&ids, !force, false);
                    force = true
                }
                println!("{id:#?}")
            }
            local.profile.remove_nth_task(&ids, !force, false);
            local.profile.order();
            local.save();
            local.show(ShowMethod::All);
        }
        1 => match args[0].as_str() {
            "-v" | "--version" => todo!(),
            "-h" | "--help" => todo!(),
            "show" => local.show(ShowMethod::All),
            _ => println!("bad command"),
        },
        2 => match args[0].as_str() {
            "tag" => local.search(&args[1]),
            "show" => match args[1].as_str() {
                "log" => local.show(ShowMethod::Log),
                "priority" => local.show(ShowMethod::Priority),
                _ => println!("bad command"),
            },
            _ => println!("bad command"),
        },
        _ => println!("bad command"),
    };
}

enum ShowMethod {
    All,
    Log,
    Priority,
}

#[derive(Debug)]
struct Local {
    profile: Profile,
    path: PathBuf,
}

impl Local {
    fn show(&self, method: ShowMethod) {
        println!("{}:", self.profile.name);
        match method {
            ShowMethod::All => {
                for (tasks, _) in &self.profile {
                    for task in tasks {
                        println!("{}", task.fmt(4, 1, true, false))
                    }
                }
            }
            ShowMethod::Log => {
                for task in filter(&self.profile.tasks, |task| {
                    matches!(&task.due, Some(due) if due.repeat != None)
                        || matches!(&task.reset_on_done, Reset::Yes(_))
                }) {
                    println!("{}", task.fmt(4, 1, false, true))
                }
            }
            ShowMethod::Priority => {}
        }
    }

    fn search(&self, str: &String) {
        for task in TagSearch::from_str(str).get(&self.profile.tasks) {
            println!("{}", task.fmt(4, 0, false, false))
        }
    }

    fn save(&self) {
        let mut path = self.path.clone();
        path.push(".lunite.json");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&self.profile.tasks).unwrap(),
        )
        .unwrap();
    }
}

enum TagSearch {
    Has(Vec<String>),
    OneOf(Vec<String>),
    Only(Vec<String>),
}

impl TagSearch {
    fn from_str(str: &String) -> Self {
        if str.chars().next() == Some('^') && str.chars().last() == Some('$') {
            let mut cp = str.to_owned();
            cp.remove(0);
            cp.pop();

            TagSearch::Only(cp.split(' ').map(|s| s.to_string()).collect())
        } else if str.contains('|') {
            TagSearch::OneOf(str.split('|').map(|s| s.to_string()).collect())
        } else {
            TagSearch::Has(str.split(' ').map(|s| s.to_string()).collect())
        }
    }

    fn get(self, tasks: &Vec<Task>) -> Vec<&Task> {
        match self {
            TagSearch::Has(tags) => TagSearch::has(tasks, tags, false),
            TagSearch::Only(tags) => TagSearch::has(tasks, tags, true),
            TagSearch::OneOf(tags) => TagSearch::one_of(tasks, tags),
        }
    }

    fn has(tasks: &Vec<Task>, tags: Vec<String>, exact: bool) -> Vec<&Task> {
        filter(tasks, |task| {
            if exact && tags.len() != task.tags.len() {
                return false;
            }

            for tag in &tags {
                if !task.tags.contains(tag) {
                    return false;
                }
            }

            true
        })
    }

    fn one_of(tasks: &Vec<Task>, tags: Vec<String>) -> Vec<&Task> {
        filter(tasks, |todo| {
            for tag in &tags {
                if todo.tags.contains(tag) {
                    return true;
                }
            }

            false
        })
    }
}
