use lunite::{Profiles, Reset};

use crate::files;

pub fn args(args: &[String]) {
    let (profiles, path) = files::get_profiles();
    let args_len = args.len();

    match args_len {
        0 => print!("{profiles}"),
        2 => match args[0].as_str() {
            "tag" => (),
            "show" => match args[1].as_str() {
                "log" => ShowMethod::Log.show(profiles),
                "priority" => ShowMethod::Priority.show(profiles),
                "links" => ShowMethod::Links.show(profiles),
                "profiles" => ShowMethod::Profiles.show(profiles),
                _ => println!("bad command {args_len} {args:?}"),
            },
            _ => println!("bad command {args_len} {args:?}"),
        },
        _ => println!("bad command {args_len} {args:?}"),
    }
}

enum ShowMethod {
    All,
    Log,
    Priority,
    Links,
    Profiles,
}

impl ShowMethod {
    fn show(&self, profiles: Profiles) {
        match self {
            ShowMethod::All => println!("{profiles}"),
            ShowMethod::Log => {
                let current = profiles.current();
                for (i, profile) in profiles.profiles().iter().enumerate() {
                    if i == current - 1 {
                        print!(">")
                    }
                    println!("{}", profile.name);
                    for task in profile.filter(|task| {
                        matches!(&task.due, Some(due) if due.repeat != None)
                            || matches!(&task.reset_on_done, Reset::Yes(_))
                    }) {
                        println!("{}", task.fmt(4, 1, false, true))
                    }
                }
            }
            _ => (),
        }
    }
}
