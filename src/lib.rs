use chrono::{Datelike, Days, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use uuid::Uuid;

macro_rules! day_creation {
    () => {
        Day {
            static_tasks: vec![],
            static_done: vec![],
            dynamic_tasks: vec![],
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum PartOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
    Fixed(TimeRange),
}

impl PartOfDay {
    pub fn fixed_from_part(&self) -> Result<Self, String> {
        Ok(Self::Fixed(match self {
            Self::Morning => {
                TimeRange::new(NaiveTime::from_hms_opt(4, 0, 0).unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            }
            Self::Afternoon => TimeRange::new(
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            ),
            Self::Evening => TimeRange::new(
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
            ),
            Self::Night => TimeRange::new(NaiveTime::from_hms_opt(21, 0, 0).unwrap(), NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            _ => return Err(format!("Didn't expect Fixed")),
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    wake_time: NaiveTime,
    bed_time: NaiveTime,
}

impl Config {
    pub fn new(wake_time: NaiveTime, bed_time: NaiveTime) -> Self {
        Self {
            wake_time,
            bed_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Planner {
    config: Config,
    days: [Day; 7],
    dynamic_tasks: Vec<DynamicTask>,
    dynamic_done: Vec<(DynamicTask, NaiveDateTime)>,
}

impl Planner {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            days: [
                day_creation!(),
                day_creation!(),
                day_creation!(),
                day_creation!(),
                day_creation!(),
                day_creation!(),
                day_creation!(),
            ],
            dynamic_tasks: vec![],
            dynamic_done: vec![],
        }
    }

    pub fn get_freetime_current(&self) -> Vec<Schedule> {
        self.current_day().get_freetime(&self.config)
    }

    pub fn get_schedule_with_dynamics(&self) -> (Vec<Schedule>, Vec<String>) {
        self.get_schedule_with_dynamics_nth(current()).unwrap()
    }

    pub fn get_freetime_nth(&self, n: usize) -> Result<Vec<Schedule>, String> {
        if n > 6 {
            return Err(format!("Expected n <= 6, n is {n}"));
        }
        Ok(self.days[n].get_freetime(&self.config))
    }

    pub fn get_schedule_with_dynamics_nth(
        &self,
        n: usize,
    ) -> Result<(Vec<Schedule>, Vec<String>), String> {
        let mut freetime = self.get_freetime_nth(n)?;
        let dynamic_tasks = self
            .current_day()
            .dynamic_tasks
            .iter()
            .map(|i| &self.dynamic_tasks[*i])
            .collect::<Vec<&DynamicTask>>();
        let mut errors = vec![];

        for task in dynamic_tasks {
            match task {
                DynamicTask::Fixed {
                    task: StaticTask { time, .. },
                    ..
                } => {
                    for i in 0..freetime.len() {
                        if let Schedule::Free(range) = &freetime[i] {
                            if time.subset(range) {
                                let first = if time.start != range.start {
                                    Some(TimeRange::new(range.start, time.start))
                                } else {
                                    None
                                };
                                let second = if time.end != range.end {
                                    Some(TimeRange::new(time.end, range.end))
                                } else {
                                    None
                                };

                                freetime[i] = Schedule::DynamicTask(task);
                                let mut ii = i;
                                if let Some(range) = first {
                                    freetime.insert(i, Schedule::Free(range));
                                    ii += 1;
                                }
                                if let Some(range) = second {
                                    freetime.insert(ii + 1, Schedule::Free(range))
                                }
                                break;
                            }
                        }
                    }
                }
                DynamicTask::Flexible {
                    length, can_split, ..
                } => {
                    if *can_split {
                        let duration = freetime
                            .iter()
                            .enumerate()
                            .filter_map(|(i, schedule)| {
                                if let Schedule::Free(range) = schedule {
                                    Some((i, range))
                                } else {
                                    None
                                }
                            })
                            .fold((vec![], Duration::seconds(0)), |mut duration, range| {
                                duration.0.push(range.0);
                                (duration.0, duration.1 + range.1.to_duration())
                            });
                        if duration.1 >= *length {
                            let mut length = length.to_owned();
                            let mut part = 1;

                            for i in duration.0 {
                                if let Schedule::Free(range) = &freetime[i] {
                                    let free_range = *range;
                                    let duration = range.to_duration();
                                    if duration == length {
                                        freetime[i] = Schedule::DynamicPart(
                                            task.fixed_split(&length, part).unwrap(),
                                        );
                                        break;
                                    } else if duration > length {
                                        freetime[i] = Schedule::DynamicPart(
                                            task.fixed_split(&length, part).unwrap(),
                                        );
                                        freetime.insert(
                                            i + 1,
                                            Schedule::Free(TimeRange::new(
                                                free_range.start + length,
                                                free_range.end,
                                            )),
                                        );

                                        break;
                                    } else {
                                        freetime[i] = Schedule::DynamicPart(
                                            task.fixed_split(&duration, part).unwrap(),
                                        );
                                        length = length - duration;
                                        part += 1;
                                    }
                                } else {
                                    unreachable!()
                                }
                            }
                        } else {
                            errors.push(format!(
                                "There isn't enough freetime for {task:#?}; total freetime {}",
                                duration.1.num_seconds()
                            ))
                        }

                        continue;
                    }

                    for i in 0..freetime.len() {
                        if let Schedule::Free(range) = &freetime[i] {
                            let range = range.to_owned();
                            let duration = range.to_duration();
                            if duration >= *length {
                                freetime[i] = Schedule::DynamicTask(task);

                                if duration != *length {
                                    freetime.insert(
                                        i + 1,
                                        Schedule::Free(TimeRange::new(
                                            range.start + *length,
                                            range.end,
                                        )),
                                    )
                                }
                                break;
                            }
                        }
                    }
                }
            };
        }

        Ok((freetime, errors))
    }

    pub fn current_day(&self) -> &Day {
        &self.days[current()]
    }

    pub fn nth_day(&self, nth: usize) -> Result<&Day, String> {
        if nth > 6 {
            return Err(format!("Expected n <= 6, n is {nth}"));
        }
        Ok(&self.days[nth])
    }

    pub fn current_day_mut(&mut self) -> &mut Day {
        &mut self.days[current()]
    }

    pub fn nth_day_mut(&mut self, nth: usize) -> Result<&mut Day, String> {
        if nth > 6 {
            return Err(format!("Expected n <= 6, n is {nth}"));
        }
        Ok(&mut self.days[nth])
    }

    pub fn add_dynamic(&mut self, task: DynamicTask) -> Result<(), String> {
        if task.get_date() < &Local::now().date_naive() {
            return Err(String::from("Task can't start in the past"));
        }

        self.dynamic_tasks.push(task);
        self.dynamic_tasks.sort();
        self.update_dynamics();
        Ok(())
    }

    pub fn complete_dynamic(&mut self, task: usize) -> Result<(), String> {
        if self.current_day_mut().dynamic_tasks.contains(&task) {
            let dynamic_id = self.current_day_mut().dynamic_tasks.swap_remove(task);
            let dynamic_task = self.dynamic_tasks.remove(dynamic_id);

            self.dynamic_done
                .push((dynamic_task, Local::now().naive_local()));
            self.update_dynamics();

            Ok(())
        } else {
            Err(format!(
                "Expected task to be in current day dynamic tasks, got {:#?}",
                self.current_day().dynamic_tasks
            ))
        }
    }

    fn update_dynamics(&mut self) {
        let mut current_date = Local::now().date_naive();

        for n in current()..7 {
            self.days[n].dynamic_tasks = self
                .dynamic_tasks
                .iter()
                .enumerate()
                .filter_map(|(i, task)| {
                    if task.get_date() == &current_date {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            current_date = current_date.checked_add_days(Days::new(1)).unwrap();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    static_tasks: Vec<StaticTask>,
    static_done: Vec<(Uuid, NaiveDateTime)>,
    dynamic_tasks: Vec<usize>,
}

impl Day {
    pub fn add_static(&mut self, task: StaticTask) {
        self.static_tasks.push(task);
        self.static_tasks.sort();
    }

    pub fn complete_static(&mut self, task: usize) -> Result<(), String> {
        if self.static_tasks.len() > task {
            let StaticTask {
                task: Task { uuid, .. },
                ..
            } = self.static_tasks[task];
            self.static_done.push((uuid, Local::now().naive_local()));

            Ok(())
        } else {
            Err(format!("There is no task at {task}"))
        }
    }

    pub fn get_freetime(&self, config: &Config) -> Vec<Schedule> {
        let times = self
            .static_tasks
            .iter()
            .filter_map(|task| {
                let StaticTask {
                    task: Task { uuid, .. },
                    ..
                } = &task;
                if self.static_done.iter().all(|t| &t.0 != uuid) {
                    Some((&task.time, task))
                } else {
                    None
                }
            })
            .collect::<Vec<(&TimeRange, &StaticTask)>>();
        let times_len = times.len();
        let mut free = vec![];

        for i in 0..times_len {
            if i == 0 {
                if config.wake_time != times[i].0.start {
                    free.push(Schedule::Free(TimeRange::new(
                        config.wake_time,
                        times[i].0.start,
                    )))
                }
                free.push(Schedule::Static(times[i].1));
                continue;
            }

            if times[i - 1].0.end != times[i].0.start {
                free.push(Schedule::Free(TimeRange::new(
                    times[i - 1].0.end,
                    times[i].0.start,
                )));
            }
            free.push(Schedule::Static(times[i].1));

            if i == times_len - 1 && times[i].0.end != config.bed_time {
                free.push(Schedule::Free(TimeRange::new(
                    times[i].0.end,
                    config.bed_time,
                )))
            }
        }

        free
    }
}

#[derive(Debug)]
pub enum Schedule<'a> {
    Static(&'a StaticTask),
    DynamicTask(&'a DynamicTask),
    DynamicPart(DynamicTask),
    Free(TimeRange),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Task {
    uuid: Uuid,
    name: String,
    description: String,
}

impl Task {
    pub fn new(name: String, description: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct StaticTask {
    task: Task,
    time: TimeRange,
}

impl Ord for StaticTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.start.cmp(&other.time.start)
    }
}

impl PartialOrd for StaticTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl StaticTask {
    pub fn new(task: Task, time: TimeRange) -> Self {
        Self { task, time }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum DynamicTask {
    Flexible {
        task: Task,
        date: NaiveDate,
        #[serde_as(as = "DurationSeconds<i64>")]
        length: Duration,
        around: PartOfDay,
        can_split: bool,
        priority: i32,
    },
    Fixed {
        task: StaticTask,
        date: NaiveDate,
        priority: i32,
    },
}

impl DynamicTask {
    pub fn new_flexible(
        task: Task,
        date: NaiveDate,
        length: Duration,
        around: PartOfDay,
        can_split: bool,
    ) -> Self {
        Self::Flexible {
            task,
            date,
            length,
            around,
            can_split,
            priority: 0,
        }
    }

    pub fn new_fixed(task: StaticTask, date: NaiveDate) -> Self {
        Self::Fixed {
            task,
            date,
            priority: 0,
        }
    }

    pub fn priority(mut self, priority: i32) -> Self {
        match &mut self {
            DynamicTask::Flexible { priority: old, .. }
            | DynamicTask::Fixed { priority: old, .. } => *old = priority,
        }
        self
    }

    fn get_date(&self) -> &NaiveDate {
        match self {
            DynamicTask::Flexible { date, .. } | DynamicTask::Fixed { date, .. } => date,
        }
    }

    fn fixed_split(&self, length: &Duration, nth: usize) -> Result<Self, String> {
        match self {
            Self::Flexible {
                task: Task {
                    name, description, ..
                },
                date,
                around,
                can_split,
                priority,
                ..
            } => Ok(Self::Flexible {
                task: Task::new(format!("{name}-{nth}"), description.to_owned()),
                date: date.to_owned(),
                length: length.to_owned(),
                around: *around,
                can_split: *can_split,
                priority: *priority,
            }),
            _ => Err(String::from("Expected a flexible dynamic task")),
        }
    }
}

impl Ord for DynamicTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Flexible { date: date1, .. }, Self::Fixed { date: date2, .. })
            | (Self::Fixed { date: date2, .. }, Self::Flexible { date: date1, .. })
                if date1 < date2 =>
            {
                std::cmp::Ordering::Less
            }
            (Self::Flexible { .. }, Self::Fixed { .. }) => std::cmp::Ordering::Greater,
            (Self::Fixed { .. }, Self::Flexible { .. }) => std::cmp::Ordering::Less,
            (
                Self::Fixed {
                    task: task1,
                    date: date1,
                    priority: priority1,
                },
                Self::Fixed {
                    task: task2,
                    date: date2,
                    priority: priority2,
                },
            ) => (date1, task1, priority1).cmp(&(date2, task2, priority2)),
            (
                Self::Flexible {
                    date: date1,
                    around: around1,
                    priority: priority1,
                    ..
                },
                Self::Flexible {
                    date: date2,
                    around: around2,
                    priority: priority2,
                    ..
                },
            ) => (date1, around1, priority1).cmp(&(date2, around2, priority2)),
        }
    }
}

impl PartialOrd for DynamicTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

impl TimeRange {
    pub fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self { start, end }
    }

    pub fn overlap(&self, other: &Self) -> bool {
        (other.start >= self.start && other.start < self.end)
            || (other.end > self.start && other.end <= self.end)
    }

    pub fn subset(&self, other: &Self) -> bool {
        (self.start >= other.start && other.start < self.end)
            && (self.end > other.start && other.end >= self.end)
    }

    pub fn to_duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}

impl Ord for TimeRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for TimeRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn current() -> usize {
    Local::now().weekday().num_days_from_monday() as usize
}
