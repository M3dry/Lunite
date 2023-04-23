use chrono::{Duration, NaiveDate, NaiveTime};
use lunite::{Config, DynamicTask, PartOfDay, Planner, StaticTask, Task, TimeRange};

fn main() {
    let mut planner = Planner::new(Config::new(
        NaiveTime::from_hms(8, 0, 0),
        NaiveTime::from_hms(21, 0, 0),
    ));
    let task1 = StaticTask::new(
        Task::new(format!("task1"), format!("")),
        TimeRange::new(NaiveTime::from_hms(8, 0, 0), NaiveTime::from_hms(10, 0, 0)),
    );
    let task2 = StaticTask::new(
        Task::new(format!("task2"), format!("")),
        TimeRange::new(NaiveTime::from_hms(10, 0, 0), NaiveTime::from_hms(12, 0, 0)),
    );
    let task3 = StaticTask::new(
        Task::new(format!("task3"), format!("")),
        TimeRange::new(NaiveTime::from_hms(13, 0, 0), NaiveTime::from_hms(14, 0, 0)),
    );

    planner.current_day_mut().add_static(task1);
    planner.current_day_mut().add_static(task2);
    planner.current_day_mut().add_static(task3);

    let dynamic_tasks = vec![
        DynamicTask::new_flexible(
            Task::new("task5".to_string(), "".to_string()),
            NaiveDate::from_ymd_opt(2023, 4, 23).unwrap(),
            Duration::minutes(120),
            PartOfDay::Night,
            false,
        ),
        DynamicTask::new_flexible(
            Task::new("task6".to_string(), "".to_string()),
            NaiveDate::from_ymd_opt(2023, 4, 23).unwrap(),
            Duration::minutes(120),
            PartOfDay::Morning,
            true,
        ),
        DynamicTask::new_fixed(
            StaticTask::new(
                Task::new("task4".to_string(), "".to_string()),
                TimeRange::new(
                    NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                ),
            ),
            NaiveDate::from_ymd_opt(2023, 4, 23).unwrap(),
        ),
        // DynamicTask::new_fixed(
        //     StaticTask::new(
        //         Task::new("task5".to_string(), "".to_string()),
        //         TimeRange::new(
        //             NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        //             NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
        //         ),
        //     ),
        //     NaiveDate::from_ymd_opt(2023, 4, 23).unwrap(),
        // ),
        DynamicTask::new_fixed(
            StaticTask::new(
                Task::new("task4".to_string(), "".to_string()),
                TimeRange::new(
                    NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
                    NaiveTime::from_hms_opt(12, 45, 0).unwrap(),
                ),
            ),
            NaiveDate::from_ymd_opt(2023, 4, 23).unwrap(),
        ),
    ];

    for task in dynamic_tasks {
        planner.add_dynamic(task).unwrap();
    }

    planner.update_dynamics();
    println!("{:#?}", planner.get_schedule_with_dynamics());
}
