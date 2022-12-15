pub mod date;

use date::Due;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Profiles {
    profiles: Vec<Profile>,
    current: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub name: String,
    todos: Vec<Todo>,
    links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Todo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<Due>,
    pub priority: i32,
    pub done: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    pub overdue: bool,
    pub index: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    pub name: String,
    pub path: PathBuf,
    local_todos: Vec<Todo>,
}

// impl Responder for Profiles {
//     type Body = BoxBody;

//     fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
//         HttpResponse::Ok()
//             .content_type(ContentType::json())
//             .body(serde_json::to_string(&self).unwrap())
//     }
// }

impl Profiles {
    pub fn new() -> Profiles {
        let mut profiles: Profiles = serde_json::from_str(
            &std::fs::read_to_string("./todos.json").unwrap_or_else(|_| {
                std::fs::write(
                    "./todos.json",
                    r#"{
  "profiles": [
    {
      "name": "Default",
      "todos": [],
      "links": []
    }
  ],
  "current": 1
}"#,
                )
                .unwrap();
                std::fs::read_to_string("./todos.json").unwrap()
            }),
        )
        .unwrap();
        profiles.overdue();
        profiles.sort();
        profiles.index();
        profiles.save();

        profiles
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn get_current(&self) -> &Profile {
        &self.profiles[self.current - 1]
    }

    pub fn get_current_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.current - 1]
    }

    pub fn get_all(&self) -> &Vec<Profile> {
        &self.profiles
    }

    pub fn get_all_mut(&mut self) -> &mut Vec<Profile> {
        &mut self.profiles
    }

    /// 1 based idx
    pub fn nth(&self, idx: usize) -> Option<&Profile> {
        self.profiles.iter().nth(idx - 1)
    }

    /// 1 based idx
    pub fn nth_mut(&mut self, idx: usize) -> Option<&mut Profile> {
        self.profiles.iter_mut().nth(idx - 1)
    }

    /// 1 based idx
    pub fn change(&mut self, idx: usize) {
        if self.profiles.len() > idx - 1 {
            self.current = idx;
        }
        self.save();
    }

    /// 1 based idx
    pub fn rename(&mut self, name: String, idx: usize) {
        if self.profiles.len() > idx - 1 {
            self.profiles[idx - 1].name = name;
        }
        self.save();
    }

    pub fn add(&mut self, profile: Profile) {
        self.profiles.push(profile);
        self.save();
    }

    /// 1 based idx
    pub fn remove(&mut self, idx: usize) {
        if self.profiles.len() > 1 {
            self.profiles.remove(idx - 1);
        }
        self.save();
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn save(&mut self) {
        self.overdue();
        self.sort();
        self.index();

        std::fs::write("./todos.json", serde_json::to_string_pretty(&self).unwrap()).unwrap();
    }

    fn overdue(&mut self) {
        for profile in &mut self.profiles {
            profile.overdue()
        }
    }

    fn index(&mut self) {
        for profile in &mut self.profiles {
            profile.index()
        }
    }

    fn sort(&mut self) {
        for profile in &mut self.profiles {
            profile.sort()
        }
    }
}

impl IntoIterator for Profile {
    type Item = (Vec<Todo>, Option<(String, PathBuf)>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut todos = vec![(self.todos, None)];

        for link in self.links {
            todos.push((link.local_todos, Some((link.name, link.path))))
        }

        todos.into_iter()
    }
}

impl<'a> IntoIterator for &'a Profile {
    type Item = (&'a Vec<Todo>, Option<(&'a String, &'a PathBuf)>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut todos = vec![(&self.todos, None)];

        for link in self.links.iter() {
            todos.push((&link.local_todos, Some((&link.name, &link.path))))
        }

        todos.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Profile {
    type Item = (&'a mut Vec<Todo>, Option<(&'a mut String, &'a mut PathBuf)>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut todos = vec![(&mut self.todos, None)];

        for link in self.links.iter_mut() {
            todos.push((
                &mut link.local_todos,
                Some((&mut link.name, &mut link.path)),
            ))
        }

        todos.into_iter()
    }
}

impl Profile {
    pub fn new(name: String) -> Profile {
        Profile {
            name,
            todos: vec![],
            links: vec![],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Vec<Todo>, Option<(&String, &PathBuf)>)> {
        self.into_iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut Vec<Todo>, Option<(&mut String, &mut PathBuf)>)> {
        self.into_iter()
    }

    /// link = 0 => global todos
    /// link = 1.. => linked todos
    pub fn add_todo(&mut self, todo: Todo, link: usize) {
        for (i, todos) in self.iter_mut().enumerate() {
            if i == link {
                todos.0.push(todo);
                break;
            }
        }

        self.index();
    }

    /// 1 based idx
    pub fn remove_todo(&mut self, idx: usize) -> Result<(), String> {
        for (todos, _) in self.iter_mut() {
            for (i, todo) in todos.iter().enumerate() {
                if idx == todo.index {
                    todos.remove(i);
                    return Ok(());
                }
            }
        }

        Err(String::from("Err: Index out of bounds"))
    }

    /// 1 based idx
    pub fn done_todo(&mut self, idxs: Vec<usize>) {
        for idx in idxs {
            for (todos, _) in self.iter_mut() {
                for todo in todos {
                    if idx == todo.index {
                        match &mut todo.due {
                            Some(due) if due.repeat => {
                                due.done();
                            }
                            _ => todo.done = !todo.done,
                        }
                    }
                }
            }
        }
    }

    pub fn remove_done_todo(&mut self) {
        self.iter_mut().for_each(|(todos, _)| {
            for (i, todo) in todos.iter().enumerate() {
                if todo.done {
                    todos.remove(i);
                    break;
                }
            }
        });
    }

    /// 1 based idx
    pub fn nth_todo(&self, idx: usize) -> Option<&Todo> {
        for (todos, _) in self {
            for todo in todos {
                if todo.index == idx {
                    return Some(todo);
                }
            }
        }

        None
    }

    /// 1 based idx
    pub fn nth_todo_mut(&mut self, idx: usize) -> Option<&mut Todo> {
        for (todos, _) in self {
            for todo in todos {
                if todo.index == idx {
                    return Some(todo);
                }
            }
        }

        None
    }

    /// 1 based idx
    pub fn replace_nth_todo(&mut self, idx: usize, replace: Todo) {
        if let Some(todo) = self.nth_todo_mut(idx) {
            *todo = replace;
        }
    }

    pub fn len_link(&self) -> usize {
        self.links.len()
    }

    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    /// 1 based idx
    pub fn remove_link(&mut self, idxs: Vec<usize>) {
        for idx in idxs {
            let mut count = 1;

            for (i, link) in self.links.iter().enumerate() {
                if count == idx {
                    self.links.remove(i);
                    break;
                }
                count += 1;
            }
        }
    }

    pub fn nth_link(&self, idx: usize) -> Option<&Link> {
        self.links.iter().nth(idx - 1)
    }

    pub fn nth_link_mut(&mut self, idx: usize) -> Option<&mut Link> {
        self.links.iter_mut().nth(idx - 1)
    }

    fn index(&mut self) {
        let mut idx = 1;

        for (todos, _) in self.iter_mut() {
            for todo in todos {
                todo.index = idx;
                idx += 1;
            }
        }
    }

    pub fn sort(&mut self) {
        self.todos.sort_by(|a, b| {
            (b.overdue, !b.done, &b.due, b.priority).cmp(&(a.overdue, !a.done, &a.due, a.priority))
        });
        for link in &mut self.links {
            link.sort();
        }
    }

    fn overdue(&mut self) {
        for (todos, _) in self {
            for todo in todos {
                match &todo.due {
                    Some(time) if time.is_overdue() && !todo.done => todo.overdue = true,
                    _ => (),
                }
            }
        }
    }
}

impl Link {
    pub fn add_todo(&mut self, todo: Todo) {
        self.local_todos.push(todo);
        self.sort();
    }

    fn sort(&mut self) {
        self.local_todos.sort_by(|a, b| {
            (b.overdue, !b.done, &b.due, b.priority).cmp(&(a.overdue, !a.done, &a.due, a.priority))
        })
    }
}
