use chrono::NaiveTime;
use lunite::{Config, Day, PartOfDay, StaticTask, Task, TimeRange};

fn main() {
    let mut day: Day =
        serde_json::from_str(r#"{ "static_tasks": [], "dynamic_tasks": [] }"#).unwrap();
    let config = &Config::new(NaiveTime::from_hms(8, 0, 0), NaiveTime::from_hms(21, 0, 0));
    let task1 = StaticTask::new(
        Task::new(format!("task1"), format!("")),
        TimeRange::new(NaiveTime::from_hms(8, 0, 0), NaiveTime::from_hms(10, 0, 0)),
    );
    let task2 = StaticTask::new(
        Task::new(format!("task2"), format!("")),
        TimeRange::new(NaiveTime::from_hms(11, 0, 0), NaiveTime::from_hms(12, 0, 0)),
    );
    let task3 = StaticTask::new(
        Task::new(format!("task3"), format!("")),
        TimeRange::new(NaiveTime::from_hms(13, 0, 0), NaiveTime::from_hms(17, 0, 0)),
    );

    day.add_static(task1);
    day.add_static(task2);
    day.add_static(task3);

    println!("{:#?}", day.get_freetime(config));
}
