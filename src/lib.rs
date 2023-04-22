use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum PartOfDay {
    Morning,
    Noon,
    Afternoon,
    Evening,
    Night,
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
}

impl Planner {
    pub fn get_freetime_current(&self) -> Vec<TimeRange> {
        self.current_day().get_freetime(&self.config)
    }

    pub fn get_freetime_nth(&self, n: usize) -> Result<Vec<TimeRange>, String> {
        if n > 6 {
            return Err(format!("Expected n <= 6, n is {n}"));
        }
        Ok(self.days[n].get_freetime(&self.config))
    }

    fn current_day(&self) -> &Day {
        &self.days[Local::now().weekday().num_days_from_monday() as usize]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    static_tasks: Vec<StaticTask>,
    dynamic_tasks: Vec<usize>,
}

impl Day {
    pub fn add_static(&mut self, task: StaticTask) {
        self.static_tasks.push(task);
        self.static_tasks.sort();
    }

    pub fn get_freetime(&self, config: &Config) -> Vec<TimeRange> {
        let times = self
            .static_tasks
            .iter()
            .map(|task| &task.time)
            .collect::<Vec<&TimeRange>>();
        let times_len = times.len();
        let mut free = vec![];

        for i in 0..times_len {
            if i == 0 {
                if config.wake_time != times[i].start {
                    free.push(TimeRange::new(config.wake_time, times[i].start))
                }
                continue;
            }

            if times[i - 1].end != times[i].start {
                free.push(TimeRange::new(times[i - 1].end, times[i].start))
            }

            if i == times_len - 1 {
                if times[i].end != config.bed_time {
                    free.push(TimeRange::new(times[i].end, config.bed_time))
                }
            }
        }

        free
    }
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
        Some(self.cmp(&other))
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
        priority: i32,
    },
    Fixed {
        task: StaticTask,
        date: NaiveDate,
        priority: i32,
    },
}

impl DynamicTask {
    pub fn new_flexible(task: Task, date: NaiveDate, length: Duration, around: PartOfDay) -> Self {
        Self::Flexible {
            task,
            date,
            length,
            around,
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
}

impl Ord for DynamicTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
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
        Some(self.cmp(&other))
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
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
}
