pub mod date;
mod tests;

use std::{fmt::Display, path::PathBuf};

use chrono::NaiveDateTime;
use date::Due;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_repr::*;

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Done {
    /// For tasks that don't have subtasks and are done
    Done = 0,
    /// For tasks that don't have subtasks and aren't done
    Undone = 1,
    /// For tasks that have subtasks which are done
    SubDone = 2,
    /// For tasks that have subtasks which aren't done
    SubUndone = 3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reset {
    /// For tasks that aren't on the top level
    NotRoot,
    /// For tasks that are on the top level, but don't reset when done
    No,
    /// For tasks that are on the top level and reset when done
    Yes(Vec<i64>),
}

impl Serialize for Reset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            Reset::NotRoot => 0.serialize(serializer),
            Reset::No => 1.serialize(serializer),
            Reset::Yes(log) => log.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Reset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResetVisitor;

        impl<'de> Visitor<'de> for ResetVisitor {
            type Value = Reset;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("an integer `0`, `1` or an array of integers between -2^63 and 2^63")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == 0 {
                    Ok(Reset::NotRoot)
                } else if v == 1 {
                    Ok(Reset::No)
                } else {
                    Err(E::custom(format!("i64 out of range: {v}")))
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == 0 {
                    Ok(Reset::NotRoot)
                } else if v == 1 {
                    Ok(Reset::No)
                } else {
                    Err(E::custom(format!("u64 out of range: {v}")))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec: Vec<i64> = vec![];
                let mut next = seq.next_element::<i64>();

                while let Ok(Some(timestamp)) = next {
                    vec.push(timestamp);
                    next = seq.next_element::<i64>();
                }
                Ok(Reset::Yes(vec))
            }
        }

        deserializer.deserialize_any(ResetVisitor)
    }
}

struct TaskFmt<'a> {
    task: &'a Task,
    indent: usize,
    level: usize,
    subtasks: bool,
    logs: bool,
}

impl<'a> TaskFmt<'a> {
    /// @hello today
    fn indent(&self) -> usize {
        self.indent * self.level
    }

    fn add(&self, levels: usize) -> usize {
        self.indent * (self.level + levels)
    }
}

impl<'a> Display for TaskFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = self.indent();

        write!(
            f,
            "{:indent$}{index}: {name}{marks}{due}{tags}{description}{logs}{subtasks}",
            "",
            name = self.task.name,
            marks = match (
                !self.task.subtasks.is_empty(),
                &self.task.done,
                &self.task.reset_on_done,
            ) {
                (true, Done::Done | Done::SubDone, Reset::Yes(_)) => "<SDR>",
                (true, Done::Done | Done::SubDone, Reset::No | Reset::NotRoot) => "<SD>",
                (true, Done::Undone | Done::SubUndone, Reset::Yes(_)) => "<SR>",
                (false, Done::Done | Done::SubDone, Reset::Yes(_)) => "<DR>",
                (true, Done::Undone | Done::SubUndone, Reset::No | Reset::NotRoot) => "<S>",
                (false, Done::Done | Done::SubDone, Reset::No | Reset::NotRoot) => "<D>",
                (false, Done::Undone | Done::SubUndone, Reset::Yes(_)) => "<R>",
                _ => "",
            },
            due = match &self.task.due {
                Some(due) =>
                    if self.task.overdue {
                        format!(": {due} - OVERDUE")
                    } else {
                        format!(": {due}")
                    },
                None => format!(""),
            },
            tags = {
                let t_string = self
                    .task
                    .tags
                    .iter()
                    .map(|t| format!(":{t}"))
                    .collect::<Vec<String>>()
                    .join("");

                if !t_string.is_empty() {
                    format!(" {t_string}:")
                } else {
                    t_string
                }
            },
            description = match &self.task.description {
                Some(description) => {
                    format!("\n{:indent$}{description}", "", indent = self.add(1))
                }
                None => format!(""),
            },
            logs = {
                if self.logs {
                    let mut buf = String::new();
                    if let Some(due) = &self.task.due {
                        if let Some(logs) = &due.repeat {
                            if !logs.is_empty() {
                                buf = format!("\n{:indent$}Log:", "", indent = self.add(1))
                            }
                            for log in logs {
                                buf = format!(
                                    "{buf}\n{:indent$}{}",
                                    "",
                                    if due.only_date {
                                        NaiveDateTime::from_timestamp(*log, 0)
                                            .date()
                                            .format("%A, %d %B %Y")
                                    } else {
                                        NaiveDateTime::from_timestamp(*log, 0)
                                            .format("%A, %d %B %H:%M:%S")
                                    },
                                    indent = self.add(2),
                                )
                            }
                        }
                    } else if let Reset::Yes(logs) = &self.task.reset_on_done {
                        if !logs.is_empty() {
                            buf = format!("\n{:indent$}Log:", "", indent = self.add(1))
                        }
                        for log in logs {
                            buf = format!(
                                "{buf}\n{:indent$}{}",
                                "",
                                NaiveDateTime::from_timestamp(*log, 0).format("%A, %d %B %H:%M:%S"),
                                indent = self.add(2),
                            )
                        }
                    }

                    buf
                } else {
                    format!("")
                }
            },
            subtasks = match &self.task.subtasks {
                _ if !self.subtasks => format!(""),
                tasks if tasks.is_empty() => format!(""),
                tasks => {
                    let mut buf = String::from("\n");

                    for task in tasks {
                        buf += &format!(
                            "{}",
                            TaskFmt {
                                task: &task,
                                indent: self.indent,
                                level: self.level + 1,
                                subtasks: self.subtasks,
                                logs: self.logs,
                            }
                        )
                    }

                    format!("{buf}")
                }
            },
            index = self.task.index,
            indent = indent
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<Due>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subtasks: Vec<Task>,
    pub reset_on_done: Reset,
    #[serde(skip_serializing_if = "is_zero", default)]
    pub priority: i32,
    pub done: Done,
    #[serde(skip_serializing_if = "is_false", default)]
    pub overdue: bool,
    pub index: usize,
}

impl Task {
    pub fn new(name: String) -> Self {
        Task {
            name,
            description: None,
            due: None,
            tags: vec![],
            subtasks: vec![],
            reset_on_done: Reset::NotRoot,
            priority: 0,
            done: Done::Undone,
            overdue: false,
            index: 0,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn due(
        mut self,
        start: i64,
        end: Option<i64>,
        date: bool,
        repeat_step: Option<Vec<i32>>,
    ) -> Self {
        let repeat = if let Some(_) = repeat_step {
            if let Reset::Yes(_) = self.reset_on_done {
                self.reset_on_done = Reset::No;
            }
            Some(vec![])
        } else {
            None
        };

        self.due = Some(Due {
            start,
            only_date: date,
            end,
            repeat,
            repeat_step,
        });
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn reset_on_done(mut self, reset: Reset) -> Self {
        self.reset_on_done = reset;
        if let Some(due) = &mut self.due {
            if due.repeat != None {
                due.repeat = None;
                due.repeat_step = None;
            }
        }
        self
    }

    pub fn done(&mut self) -> Result<(), String> {
        self.done = match self.done {
            Done::Undone => {
                if let Reset::Yes(log) = &mut self.reset_on_done {
                    log.push(date::timestamp());
                    Done::Undone
                } else if let Some(due) = &mut self.due {
                    if due.repeat != None {
                        due.done();
                        Done::Undone
                    } else {
                        Done::Done
                    }
                } else {
                    Done::Done
                }
            }
            Done::Done => Done::Undone,
            _ => return Err(String::from("Err: Taks has subtasks")),
        };

        Ok(())
    }

    pub fn add_subtask(&mut self, mut task: Task) {
        task.reset_on_done = Reset::NotRoot;
        if matches!(task.done, Done::Done | Done::SubDone) {
            task.done();
        }
        self.subtasks.push(task);
        sort(&mut self.subtasks);
    }

    pub fn iter<F, T>(&self, func: F, t: &mut T)
    where
        F: Fn(&Vec<Task>, usize, &mut T) + Copy,
        T: Copy,
    {
        func(&self.subtasks, self.index, t);

        for task in &self.subtasks {
            if !task.subtasks.is_empty() {
                task.iter(func, t);
            }
        }
    }

    pub fn iter_mut<F, T>(&mut self, func: F, t: &mut T)
    where
        F: Fn(&mut Vec<Task>, usize, &mut T) + Copy,
        T: Copy,
    {
        func(&mut self.subtasks, self.index, t);

        for task in &mut self.subtasks {
            if !task.subtasks.is_empty() {
                task.iter_mut(func, t);
            }
        }
    }

    pub fn fmt(&self, indent: usize, level: usize, subtasks: bool, logs: bool) -> String {
        format!(
            "{}",
            TaskFmt {
                task: self,
                indent,
                level,
                subtasks,
                logs
            }
        )
    }

    /// 1 based idx
    pub fn nth_task(&self, idx: usize) -> Result<&Task, String> {
        fn iter(task: &Task, idx: usize) -> Option<&Task> {
            if task.index == idx {
                return Some(task);
            }

            for subtask in &task.subtasks {
                if subtask.index == idx {
                    return Some(subtask);
                } else if !subtask.subtasks.is_empty() {
                    return iter(subtask, idx);
                }
            }

            None
        }

        if self.index == idx {
            return Ok(self);
        }

        for task in &self.subtasks {
            match iter(task, idx) {
                Some(task) => return Ok(task),
                None => continue,
            }
        }

        Err(String::from("Err: Idx out of bounds"))
    }

    /// 1 based idx
    pub fn nth_task_mut(&mut self, idx: usize) -> Result<&mut Task, String> {
        fn iter(task: &mut Task, idx: usize) -> Option<&mut Task> {
            if task.index == idx {
                return Some(task);
            }

            for subtask in &mut task.subtasks {
                if subtask.index == idx {
                    return Some(subtask);
                } else if !subtask.subtasks.is_empty() {
                    return iter(subtask, idx);
                }
            }

            None
        }

        if self.index == idx {
            return Ok(self);
        }

        for task in &mut self.subtasks {
            match iter(task, idx) {
                Some(task) => return Ok(task),
                None => continue,
            }
        }

        Err(String::from("Err: Idx out of bounds"))
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            TaskFmt {
                task: self,
                indent: 4,
                level: 0,
                subtasks: true,
                logs: false,
            }
        )
    }
}

pub struct LinkFmt<'a> {
    link: &'a Link,
    indent: usize,
    level: usize,
    subtasks: bool,
    logs: bool,
}

impl<'a> LinkFmt<'a> {
    fn indent(&self) -> usize {
        self.indent * self.level
    }
}

impl<'a> Display for LinkFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:indent$}{name}({path:#?}):\n{tasks}",
            "",
            name = self.link.name,
            path = self.link.path,
            tasks = match self
                .link
                .tasks
                .iter()
                .map(|task| format!(
                    "{}",
                    TaskFmt {
                        task,
                        indent: self.indent,
                        level: self.level + 1,
                        subtasks: self.subtasks,
                        logs: self.logs
                    }
                ))
                .collect::<Vec<String>>()
                .join("")
            {
                tasks if tasks.is_empty() => tasks,
                tasks => tasks + "\n",
            },
            indent = self.indent()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Link {
    name: String,
    path: PathBuf,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tasks: Vec<Task>,
    #[serde(skip)]
    exists: bool,
}

impl Link {
    pub fn new(name: String, path: PathBuf, tasks: Vec<Task>) -> Link {
        Link {
            name,
            path,
            tasks,
            exists: false,
        }
    }

    pub fn name(&mut self, name: String) {
        self.name = name;
    }

    pub fn path(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn add_task(&mut self, mut task: Task) {
        if task.reset_on_done == Reset::NotRoot {
            task.reset_on_done = Reset::No;
        }
        self.tasks.push(task);
        sort(&mut self.tasks);
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            LinkFmt {
                link: self,
                indent: 4,
                level: 0,
                subtasks: true,
                logs: false
            }
        )
    }
}

struct ProfileFmt<'a> {
    profile: &'a Profile,
    indent: usize,
    level: usize,
    subtasks: bool,
    logs: bool,
}

impl<'a> ProfileFmt<'a> {
    fn indent(&self) -> usize {
        self.indent * self.level
    }
}

impl<'a> Display for ProfileFmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:indent$}{name}:\n{tasks}",
            "",
            name = self.profile.name,
            tasks = {
                let tasks = self
                    .profile
                    .tasks
                    .iter()
                    .map(|task| {
                        format!(
                            "{}",
                            TaskFmt {
                                task,
                                indent: self.indent,
                                level: self.level + 1,
                                subtasks: self.subtasks,
                                logs: self.logs,
                            }
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("");
                let links = self
                    .profile
                    .links
                    .iter()
                    .map(|link| {
                        format!(
                            "{}",
                            LinkFmt {
                                link,
                                indent: self.indent,
                                level: self.level + 1,
                                subtasks: self.subtasks,
                                logs: self.logs,
                            }
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("");

                if links.is_empty() {
                    tasks
                } else if tasks.is_empty() {
                    links
                } else {
                    tasks + "\n\n" + &links
                }
            },
            indent = self.indent()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tasks: Vec<Task>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<Link>,
}

impl Profile {
    pub fn new(name: String, tasks: Vec<Task>, mut links: Vec<Link>) -> Profile {
        for link in &mut links {
            link.exists = false;
        }
        Profile { name, tasks, links }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Vec<Task>, Option<(&String, &PathBuf)>)> {
        self.into_iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut Vec<Task>, Option<(&mut String, &mut PathBuf)>)> {
        self.into_iter()
    }

    pub fn filter<F>(&self, condition: F) -> Vec<&Task>
    where
        F: Fn(&Task) -> bool + Copy,
    {
        self.iter()
            .map(|(tasks, _)| filter(tasks, condition))
            .flatten()
            .collect()
    }

    /// 1 based idx
    pub fn nth_task(&self, idx: usize) -> Result<&Task, String> {
        for (tasks, _) in self {
            for task in tasks {
                match task.nth_task(idx) {
                    Ok(task) => return Ok(task),
                    Err(_) => continue,
                }
            }
        }

        Err(String::from("Err: Idx out of bounds"))
    }

    /// 1 based idx
    pub fn nth_task_mut(&mut self, idx: usize) -> Result<&mut Task, String> {
        for (tasks, _) in self.iter_mut() {
            for task in tasks {
                match task.nth_task_mut(idx) {
                    Ok(task) => return Ok(task),
                    Err(_) => continue,
                }
            }
        }

        Err(String::from("Err: Idx out of bounds"))
    }

    pub fn root_nth_task_idx(&self, idx: usize) -> Result<usize, String> {
        fn iter<'a>(
            parent: Option<usize>,
            tasks: &'a Vec<Task>,
            idx: usize,
        ) -> Result<usize, bool> {
            for task in tasks {
                if task.index == idx {
                    return match parent {
                        Some(task) => Ok(task),
                        None => Err(true),
                    };
                } else if !task.subtasks.is_empty() {
                    return iter(Some(task.index), &task.subtasks, idx);
                }
            }

            Err(false)
        }

        for (tasks, _) in self {
            match iter(None, tasks, idx) {
                Ok(task) => return Ok(task),
                Err(true) => return Err(String::from("Err: Task doesn't have a parent")),
                _ => (),
            }
        }

        Err(String::from("Err: Index out of bounds"))
    }

    /// 1 based idx
    pub fn remove_nth_task(&mut self, idxs: &Vec<usize>, preserve_subs: bool, order: bool) {
        fn iter(
            tasks: &mut Task,
            idx: usize,
            mut subs: Option<Vec<Task>>,
        ) -> (bool, Option<Vec<Task>>) {
            for (i, task) in tasks.subtasks.iter_mut().enumerate() {
                if task.index == idx {
                    if subs != None {
                        subs = Some(std::mem::replace(&mut task.subtasks, vec![]));
                    } else {
                        tasks.subtasks.remove(i);
                    }
                    return (true, subs);
                } else {
                    subs = match iter(task, idx, subs) {
                        (true, subs) => return (true, subs),
                        (false, vec) => vec,
                    }
                }
            }

            return (false, subs);
        }

        let mut subs: Vec<Task> = vec![];
        for idx in idxs {
            'f: for (tasks, _) in self.iter_mut() {
                for (i, task) in tasks.iter_mut().enumerate() {
                    if task.index == *idx {
                        if preserve_subs {
                            subs = std::mem::replace(&mut task.subtasks, vec![]);
                        }
                        if subs.is_empty() {
                            tasks.remove(i);
                        }
                        break 'f;
                    }

                    subs = match iter(task, *idx, if preserve_subs { Some(subs) } else { None }) {
                        (true, vec) => {
                            subs = match vec {
                                Some(subs) => subs,
                                None => vec![],
                            };
                            break 'f;
                        }
                        _ => vec![],
                    };
                }
            }

            if preserve_subs {
                let root = match self.nth_task_mut(match self.root_nth_task_idx(*idx) {
                    Ok(idx) => idx,
                    Err(_) => continue,
                }) {
                    Ok(root) => root,
                    Err(_) => continue,
                };

                for (i, task) in root.subtasks.iter().enumerate() {
                    if task.index == *idx {
                        root.subtasks.remove(i);
                        break;
                    }
                }
                root.subtasks
                    .append(&mut std::mem::replace(&mut subs, vec![]));
            }
        }

        if order {
            self.order();
        }
    }

    /// 1 based idx
    pub fn replace_nth_task(&mut self, idx: usize, replace: Task) -> Result<(), String> {
        let ret = self.nth_task_mut(idx).map(|x| *x = replace);
        self.order();
        ret
    }

    /// link = 0 => global tasks
    /// link = 1.. => linked tasks
    pub fn add_task(&mut self, mut task: Task, link: usize) -> Result<(), String> {
        if task.reset_on_done == Reset::NotRoot {
            task.reset_on_done = Reset::No;
        }

        for (i, tasks) in self.iter_mut().enumerate() {
            if i == link {
                tasks.0.push(task);
                break;
            }
        }

        self.order();
        Ok(())
    }

    /// 1 based idx
    pub fn add_subtask(&mut self, mut task: Task, master_idx: usize) -> Result<(), String> {
        task.reset_on_done = Reset::NotRoot;
        let ret = self.nth_task_mut(master_idx).map(|master| {
            master.done();
            master.subtasks.push(task)
        });
        self.order();
        ret
    }

    /// 1 based idx
    pub fn done(&mut self, idx: usize) -> Result<(), String> {
        fn upcheck_done(profile: &mut Profile, idx: usize) {
            let mut all = true;
            let task = match profile.nth_task_mut(idx) {
                Err(_) => return,
                Ok(task) => task,
            };

            for subtask in &task.subtasks {
                match subtask.done {
                    Done::Done | Done::SubDone => (),
                    _ => {
                        all = false;
                    }
                }
            }

            if all {
                task.done = Done::SubDone;
                match &mut task.reset_on_done {
                    Reset::Yes(log) => {
                        task.done = Done::SubUndone;
                        log.push(date::timestamp());
                        task.iter_mut(
                            |tasks, _, _| {
                                for task in tasks {
                                    task.done = match task.done {
                                        Done::SubDone => Done::SubUndone,
                                        Done::Done => Done::Undone,
                                        _ => panic!("this shouldn't happen"),
                                    }
                                }
                            },
                            &mut 0,
                        )
                    }
                    Reset::No => (),
                    Reset::NotRoot => match profile.root_nth_task_idx(idx) {
                        Err(_) => (),
                        Ok(idx) => upcheck_done(profile, idx),
                    },
                };
            }
        }

        let task = self.nth_task_mut(idx)?;
        task.done()?;
        match task.reset_on_done {
            Reset::NotRoot => upcheck_done(self, self.root_nth_task_idx(idx)?),
            _ => (),
        }

        self.order();
        Ok(())
    }

    pub fn remove_done(&mut self) {
        self.remove_nth_task(
            &self
                .iter()
                .map(|(tasks, _)| {
                    tasks
                        .iter()
                        .filter(|task| {
                            task.reset_on_done == Reset::No
                                && (task.done == Done::Done || task.done == Done::SubDone)
                        })
                        .map(|task| task.index)
                        .collect::<Vec<usize>>()
                })
                .flatten()
                .collect::<Vec<usize>>(),
            false,
            true,
        );
    }

    pub fn link_len(&self) -> usize {
        self.links.len()
    }

    pub fn add_link(&mut self, mut link: Link) {
        link.exists = false;
        self.links.push(link);
    }

    /// 1 based idx
    pub fn nth_link(&self, idx: usize) -> Result<&Link, String> {
        match self.links.iter().nth(idx - 1) {
            Some(link) => Ok(link),
            None => Err(String::from("Err: Idx ouf of bounds")),
        }
    }

    /// 1 based idx
    pub fn nth_link_mut(&mut self, idx: usize) -> Result<&mut Link, String> {
        match self.links.iter_mut().nth(idx - 1) {
            Some(link) => Ok(link),
            None => Err(String::from("Err: Idx ouf of bounds")),
        }
    }

    /// 1 based idx
    pub fn remove_nth_link(&mut self, idx: usize) -> Result<(), String> {
        if self.link_len() <= idx {
            Err(String::from("Err: Idx out of bounds"))
        } else {
            self.links.remove(idx - 1);
            Ok(())
        }
    }

    /// 1 based idx
    pub fn replace_nth_link(&mut self, idx: usize, mut new: Link) -> Result<(), String> {
        new.exists = false;
        self.nth_link_mut(idx).map(|link| *link = new)
    }

    pub fn order(&mut self) {
        fn change_overdue(task: &mut Task) {
            match &task.due {
                Some(time) if time.is_overdue() && task.done != Done::Done => task.overdue = true,
                Some(time) => task.overdue = false,
                None if task.overdue => task.overdue = false,
                _ => ()
            }
        }
        let mut idx = 1;

        for (tasks, _) in self.iter_mut() {
            for task in tasks.iter_mut() {
                change_overdue(task);
                task.iter_mut(
                    |ts, _, _| {
                        for t in ts {
                            change_overdue(t);
                        }
                    },
                    &mut 0,
                );
            }
            sort(tasks);
            for task in tasks.iter_mut() {
                task.index = idx;
                idx += 1;
                task.iter_mut(
                    |ts, _, idx| {
                        for t in ts {
                            t.index = *idx;
                            *idx += 1;
                        }
                    },
                    &mut idx,
                )
            }
        }
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            ProfileFmt {
                profile: self,
                indent: 4,
                level: 0,
                subtasks: true,
                logs: false
            }
        )
    }
}

impl<'a> IntoIterator for &'a Profile {
    type Item = (&'a Vec<Task>, Option<(&'a String, &'a PathBuf)>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut ret = vec![(&self.tasks, None)];

        for link in &self.links {
            ret.push((&link.tasks, Some((&link.name, &link.path))))
        }

        ret.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Profile {
    type Item = (&'a mut Vec<Task>, Option<(&'a mut String, &'a mut PathBuf)>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut ret = vec![(&mut self.tasks, None)];

        for link in &mut self.links {
            ret.push((&mut link.tasks, Some((&mut link.name, &mut link.path))))
        }

        ret.into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profiles {
    profiles: Vec<Profile>,
    /// 1 based idx
    current: usize,
}

impl Profiles {
    pub fn new(mut profiles: Vec<Profile>) -> Self {
        if profiles.len() == 0 {
            profiles.push(Profile::new(String::from("Default"), vec![], vec![]));
        }
        Self {
            profiles,
            current: 1,
        }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// 1 based idx
    pub fn change_current(&mut self, idx: usize) {
        self.current = idx;
    }

    pub fn current_profile(&self) -> &Profile {
        &self.profiles[self.current - 1]
    }

    pub fn current_profile_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.current - 1]
    }

    pub fn profiles(&self) -> &Vec<Profile> {
        &self.profiles
    }

    pub fn profiles_mut(&mut self) -> &mut Vec<Profile> {
        &mut self.profiles
    }

    /// 1 based idx
    pub fn nth(&self, idx: usize) -> Result<&Profile, String> {
        if self.len() < idx {
            Err(String::from("Err: Idx out of bounds"))
        } else {
            Ok(&self.profiles[idx - 1])
        }
    }

    /// 1 based idx
    pub fn nth_mut(&mut self, idx: usize) -> Result<&mut Profile, String> {
        if self.len() < idx {
            Err(String::from("Err: Idx out of bounds"))
        } else {
            Ok(&mut self.profiles[idx - 1])
        }
    }

    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
        self.order()
    }

    /// 1 based idx
    pub fn remove_profile(&mut self, idx: usize) -> Result<(), String> {
        if self.len() < idx {
            Err(String::from("Err: Idx out of bounds"))
        } else {
            self.profiles.remove(idx - 1);
            Ok(self.order())
        }
    }

    /// 1 based idx
    pub fn rename_profile(&mut self, idx: Option<usize>, name: String) -> Result<(), String> {
        if let Some(idx) = idx {
            self.nth_mut(idx)?.name = name;
        } else {
            self.current_profile_mut().name = name;
        }

        Ok(self.order())
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn order(&mut self) {
        for profile in &mut self.profiles {
            profile.order();
        }
    }
}

impl Display for Profiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{profiles}",
            profiles = self
                .profiles
                .iter()
                .enumerate()
                .map(|(i, profile)| if i + 1 == self.current && i != 0 {
                    format!("\n>{profile}")
                } else if i + 1 == self.current {
                    format!(">{profile}")
                } else {
                    format!("\n{profile}")
                })
                .collect::<Vec<String>>()
                .join("")
        )
    }
}

pub fn filter<F>(tasks: &Vec<Task>, condition: F) -> Vec<&Task>
where
    F: Fn(&Task) -> bool + Copy,
{
    fn iter<'a, 'b, F>(task: &'a Task, tasks: &'b mut Vec<&'a Task>, condition: F)
    where
        F: Fn(&'a Task) -> bool + Copy,
    {
        if condition(task) {
            tasks.push(task)
        }

        for subtask in &task.subtasks {
            iter(subtask, tasks, condition);
        }
    }

    let mut ret = vec![];

    for task in tasks {
        iter(task, &mut ret, condition);
    }

    ret
}

fn sort(tasks: &mut Vec<Task>) {
    tasks.sort_by(|a, b| {
        (
            b.overdue
                ^ match a.done {
                    Done::Done | Done::SubDone => true,
                    Done::Undone | Done::SubUndone => false,
                },
            &b.due,
            b.priority,
        )
            .cmp(&(
                a.overdue
                    ^ match b.done {
                        Done::Done | Done::SubDone => true,
                        Done::Undone | Done::SubUndone => false,
                    },
                &a.due,
                a.priority,
            ))
    });

    for task in tasks {
        if !task.subtasks.is_empty() {
            sort(&mut task.subtasks);
        }
    }
}

fn is_zero(n: &i32) -> bool {
    n == &0
}

fn is_false(bool: &bool) -> bool {
    bool == &false
}
