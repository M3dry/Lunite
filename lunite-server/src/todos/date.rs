use chrono::{Local, NaiveDateTime, Duration, NaiveDate, Datelike};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Due {
    pub time: (NaiveDateTime, Option<NaiveDateTime>),
    pub only_date: bool,
    pub repeat: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weeks: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub months: Option<i32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub log: Vec<NaiveDateTime>,
}

impl PartialEq for Due {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl Eq for Due {}

impl PartialOrd for Due {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time.partial_cmp(&other.time)
    }
}

impl Ord for Due {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl Due {
    pub fn is_overdue(&self) -> bool {
        let now = Local::now().naive_local();
        let time = match self.time.1 {
            Some(time) => time,
            None => self.time.0,
        };

        time < now
    }

    pub fn done(&mut self) {
        let curr = Local::now().naive_local();

        while self.get_time() < &curr {
            self.advance();
        }
        self.log.push(curr);
    }

    fn get_time(&self) -> &NaiveDateTime {
        match &self.time.1 {
            Some(time) => time,
            None => &self.time.0,
        }
    }

    fn advance(&mut self) {
        let duration = self.to_duration();
        self.time.0 += duration;
        match &mut self.time.1 {
            Some(time) => {
                *time += duration;
            },
            None => (),
        };
    }

    fn to_duration(&self) -> Duration {
        let mut duration = match &self.hours {
            Some(hours) => Duration::hours(*hours as i64),
            None => Duration::hours(0),
        };

        duration = duration
            + match &self.days {
                Some(days) => Duration::days(*days as i64),
                None => Duration::hours(0),
            };

        duration = duration
            + match &self.weeks {
                Some(weeks) => Duration::weeks(*weeks as i64),
                None => Duration::hours(0),
            };

        duration = duration
            + match &self.months {
                Some(months) => {
                    let now = Local::now();
                    Duration::days(*months as i64 * days_from_month(now.year(), now.month()))
                }
                None => Duration::hours(0),
            };

        duration
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum Due {
//     Time(Time),
//     Repeat(Repeat),
// }

// impl std::fmt::Display for Due {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match &self {
//                 Due::Time(time) => format!("{}", time),
//                 Due::Repeat(repeat) => format!("{}", repeat),
//             }
//         )
//     }
// }

// impl PartialEq for Due {
//     fn eq(&self, other: &Self) -> bool {
//         self.get_time().eq(&other.get_time())
//     }
// }

// impl Eq for Due {}

// impl PartialOrd for Due {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         other.get_time().partial_cmp(&other.get_time())
//     }
// }

// impl Ord for Due {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.get_time().cmp(&other.get_time())
//     }
// }

// impl Due {
//     pub fn is_overdue(&self) -> bool {
//         if self.get_time().get_end() < Local::now().naive_local() {
//             true
//         } else {
//             false
//         }
//     }

//     pub fn is_date(&self) -> bool {
//         match self {
//             Due::Repeat(repeat) => repeat.time.is_date(),
//             Due::Time(time) => time.is_date(),
//         }
//     }

//     pub fn get_time(&self) -> &Time {
//         match self {
//             Due::Time(time) => time,
//             Due::Repeat(repeat) => &repeat.time,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum Time {
//     Time(NaiveDateTime),
//     Date(NaiveDate),
//     RangeTime(NaiveDateTime, NaiveDateTime),
//     RangeDate(NaiveDate, NaiveDate),
// }

// impl std::fmt::Display for Time {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Time::Time(datetime) => {
//                 write!(f, "{}", datetime.format("%A, %d %B %H:%M:%S"))
//             }
//             Time::Date(date) => {
//                 write!(f, "{}", date.format("%A, %d %B %Y"))
//             }
//             Time::RangeTime(start, end) => {
//                 write!(
//                     f,
//                     "{} <-> {}",
//                     start.format("%A, %d %B %H:%M:%S"),
//                     end.format("%A, %d %B %H:%M:%S")
//                 )
//             }
//             Time::RangeDate(start, end) => {
//                 write!(
//                     f,
//                     "{} <-> {}",
//                     start.format("%A, %d %B %Y"),
//                     end.format("%A, %d %B %Y")
//                 )
//             }
//         }
//     }
// }

// impl PartialEq for Time {
//     fn eq(&self, other: &Self) -> bool {
//         if self.get_start() == other.get_start() {
//             true
//         } else {
//             false
//         }
//     }

//     fn ne(&self, other: &Self) -> bool {
//         !self.eq(other)
//     }
// }

// impl Eq for Time {}

// impl PartialOrd for Time {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         other.get_start().partial_cmp(&self.get_start())
//     }
// }

// impl Ord for Time {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         other.get_start().cmp(&self.get_start())
//     }
// }

// impl Time {
//     pub fn get_end(&self) -> NaiveDateTime {
//         match self {
//             Time::Time(time) => *time,
//             Time::Date(date) => date.and_hms(0, 0, 0),
//             Time::RangeTime(_, end) => *end,
//             Time::RangeDate(_, end) => end.and_hms(0, 0, 0),
//         }
//     }

//     pub fn get_start(&self) -> NaiveDateTime {
//         match self {
//             Time::Time(time) => *time,
//             Time::Date(date) => date.and_hms(0, 0, 0),
//             Time::RangeTime(start, _) => *start,
//             Time::RangeDate(start, _) => start.and_hms(0, 0, 0),
//         }
//     }

//     fn is_date(&self) -> bool {
//         match self {
//             Time::Time(_) | Time::RangeTime(_, _) => false,
//             Time::Date(_) | Time::RangeDate(_, _) => true,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Repeat {
//     pub time: Time,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub hours: Option<i32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub days: Option<i32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub weeks: Option<i32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub months: Option<i32>,
//     #[serde(skip_serializing_if = "Vec::is_empty", default)]
//     pub log: Vec<NaiveDateTime>,
// }

// impl std::fmt::Display for Repeat {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let repeat = match &self.hours {
//             Some(hours) => format!("{hours}h"),
//             None => String::new(),
//         };
//         let repeat = match &self.days {
//             Some(days) => format!("{repeat}{days}d"),
//             None => repeat,
//         };
//         let repeat = match &self.weeks {
//             Some(weeks) => format!("{repeat}{weeks}w"),
//             None => repeat,
//         };
//         let repeat = match &self.months {
//             Some(months) => format!("{repeat}{months}m"),
//             None => repeat,
//         };

//         write!(
//             f,
//             "{}{}",
//             self.time,
//             if repeat.is_empty() {
//                 format!("")
//             } else {
//                 format!(": {repeat}")
//             }
//         )
//     }
// }

// impl Repeat {
//     pub fn done(&mut self) {
//         let curr = Local::now().naive_local();

//         while self.time.get_end() < curr {
//             self.advance();
//         }
//         self.log.push(curr);
//     }

//     fn advance(&mut self) {
//         let duration = self.to_duration();
//         match &mut self.time {
//             Time::Time(time) => *time += duration,
//             Time::Date(date) => *date += duration,
//             Time::RangeTime(start, end) => {
//                 *start += duration;
//                 *end += duration;
//             }
//             Time::RangeDate(start, end) => {
//                 *start += duration;
//                 *end += duration;
//             }
//         };
//     }

//     fn to_duration(&self) -> Duration {
//         let mut duration = match &self.hours {
//             Some(hours) => Duration::hours(*hours as i64),
//             None => Duration::hours(0),
//         };

//         duration = duration
//             + match &self.days {
//                 Some(days) => Duration::days(*days as i64),
//                 None => Duration::hours(0),
//             };

//         duration = duration
//             + match &self.weeks {
//                 Some(weeks) => Duration::weeks(*weeks as i64),
//                 None => Duration::hours(0),
//             };

//         duration = duration
//             + match &self.months {
//                 Some(months) => {
//                     let now = Local::now();
//                     Duration::days(*months as i64 * days_from_month(now.year(), now.month()))
//                 }
//                 None => Duration::hours(0),
//             };

//         duration
//     }
// }

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
