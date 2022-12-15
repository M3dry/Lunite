use std::fmt::Display;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Due {
    pub start: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    pub only_date: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_step: Option<Vec<i32>>,
}

impl Display for Due {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{start}{end}{repeat}",
            start = if self.only_date {
                NaiveDateTime::from_timestamp(self.start, 0)
                    .date()
                    .format("%A, %d %B %Y")
            } else {
                NaiveDateTime::from_timestamp(self.start, 0).format("%A, %d %B %H:%M:%S")
            },
            end = match self.end {
                Some(end) => format!(
                    " <-> {}",
                    if self.only_date {
                        NaiveDateTime::from_timestamp(end, 0)
                            .date()
                            .format("%A, %d %B %Y")
                    } else {
                        NaiveDateTime::from_timestamp(end, 0).format("%A, %d %B %H:%M:%S")
                    }
                ),
                None => format!(""),
            },
            repeat = match &self.repeat_step {
                Some(repeat_step) => {
                    format!(" - {}", repeat_step.iter().enumerate().filter_map(|(i, r)| {
                        if *r != 0 {
                            Some(format!("{}{}", r, match i {
                                0 => "h",
                                1 => "d",
                                2 => "w",
                                3 => "m",
                                x => panic!("{x} how?"),
                            }))
                        } else {
                            None
                        }
                    }).collect::<Vec<String>>().join(""))
                }
                None => format!(""),
            }
        )
    }
}

impl PartialEq for Due {
    fn eq(&self, other: &Self) -> bool {
        (self.start, self.end).eq(&(other.start, other.end))
    }
}

impl Eq for Due {}

impl PartialOrd for Due {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (other.start, other.end).partial_cmp(&(self.start, self.end))
    }
}

impl Ord for Due {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.start, other.end).cmp(&(self.start, self.end))
    }
}

impl Due {
    pub fn is_overdue(&self) -> bool {
        (match self.end {
            Some(time) => time,
            None => self.start,
        }) < timestamp()
    }

    pub fn log_push(&mut self, timestamp: i64) {
        if let Some(log) = &mut self.repeat {
            log.push(timestamp)
        }
    }

    pub fn done(&mut self) {
        let curr = timestamp();

        while self.get_time() < curr {
            self.advance();
        }
        self.log_push(curr);
    }

    fn get_time(&self) -> i64 {
        match self.end {
            Some(time) => time,
            None => self.start,
        }
    }

    fn advance(&mut self) {
        let duration = self.to_duration();
        self.start += duration;
        match &mut self.end {
            Some(time) => {
                *time += duration;
            }
            None => (),
        };
    }

    fn to_duration(&self) -> i64 {
        let step_secs: [i64; 3] = [3600, 86400, 604800];

        match &self.repeat_step {
            Some(steps) => {
                let mut duration = 0 as i64;

                for (i, step) in steps.iter().enumerate() {
                    if i < 3 {
                        duration += *step as i64 * step_secs[i]
                    } else {
                        let now = Local::now();
                        let mut year = now.year();
                        let mut month = now.month();

                        for _ in 0..*step {
                            duration += days_from_month(year, month) * step_secs[1];
                            if month == 12 {
                                year += 1;
                                month = 0;
                            }
                            month += 1;
                        }
                        break;
                    }
                }

                duration
            }
            None => 0,
        }
    }
}

fn days_from_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days()
}

pub fn timestamp() -> i64 {
    Local::now().timestamp()
}
