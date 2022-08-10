use chrono::{DateTime, Duration};
use chrono::{Datelike, Local, TimeZone, Timelike, Utc};
use chrono_tz::Europe::London;
use chrono_tz::Tz;
use chrono_tz::{OffsetComponents, OffsetName};
use std::cmp::Ordering;

#[derive(Default, Debug)]
struct Intervals {
    work_min: u32,
    rest_min: u32,
}

#[derive(Default, Debug)]
struct Target {
    hour: u32,
    min: u32,
    sec: u32,
}

#[derive(Default, Debug)]
enum ScheduleError {
    #[default]
    NotInTheFuture,
    InvalidHourMinSec,
    IntervalGreaterThanAvailableTime,
}

impl Target {
    pub fn hour(hour: u32) -> Self {
        Self {
            hour,
            ..Default::default()
        }
    }
    pub fn hour_min(hour: u32, min: u32) -> Self {
        Self {
            hour,
            min,
            ..Default::default()
        }
    }
}

fn main() -> Result<(), ScheduleError> {
    let now_time: DateTime<chrono_tz::Tz> = Utc::now().with_timezone(&London);
    let target = Target::hour_min(16, 0);
    let iters = create_schedule(&now_time, &target)?;
    let mut running_total = 0;
    for ts in iters.timetable.entries {
        println!(
            "⏱{} {:?}={}",
            running_total,
            ts.kind,
            ts.duration.num_minutes()
        );
        running_total += ts.duration.num_minutes();
    }

    // for time_segment in iters.sequence {
    //     println!("{:?}", time_segment.duration.num_minutes());
    // }

    // dbg!(diff.num_minutes());
    // let left_over = diff.num_minutes() / interval_target.num_minutes();
    // // dbg!(interval_target);
    // dbg!(left_over);
    // println!("diff: ({:02}:{:02}:{:02})",
    //          dur_to_target.num_hours(),
    //          dur_to_target.num_minutes() % 60,
    //          dur_to_target.num_seconds() % 60);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum ActivityKind {
    Work,
    Rest,
}

#[derive(Debug)]
struct Activity {
    duration: Duration,
    kind: ActivityKind,
}

#[derive(Debug)]
struct Timetable {
    entries: Vec<Activity>,
}

#[derive(Debug)]
struct Schedule {
    timetable: Timetable,
    start_time: DateTime<chrono_tz::Tz>,
    end_time: DateTime<chrono_tz::Tz>,
    remaining: Duration,
}

fn create_schedule(
    now_time: &DateTime<chrono_tz::Tz>,
    target: &Target,
) -> Result<Schedule, ScheduleError> {
    let interval_target = Duration::minutes(30);
    let rest_interval = Duration::minutes(5);
    let (dur_to_target, end_time) = get_duration_until(now_time, target)?;
    match dur_to_target
        .num_minutes()
        .cmp(&interval_target.num_minutes())
    {
        Ordering::Less => {
            println!("available time was less than interval time");
            Err(ScheduleError::IntervalGreaterThanAvailableTime)
        }
        Ordering::Equal => {
            println!("available time was equal to interval time");
            Err(ScheduleError::IntervalGreaterThanAvailableTime)
        }
        Ordering::Greater => {
            let iterations = dur_to_target.num_minutes() / interval_target.num_minutes();
            // let end_time = interval_target.num_minutes() * iterations;
            // let as_d = Duration::minutes(end_time);
            // let ending: DateTime<chrono_tz::Tz> = now_time + as_d;
            let remaining_mins = dur_to_target.num_minutes() % interval_target.num_minutes();
            let mut entries: Vec<Activity> = (0..iterations)
                .map(|num| {
                    vec![
                        Activity {
                            duration: interval_target - rest_interval,
                            kind: ActivityKind::Work,
                        },
                        Activity {
                            duration: rest_interval,
                            kind: ActivityKind::Rest,
                        },
                    ]
                })
                .flatten()
                .collect();
            if remaining_mins > 0 {
                entries.push(Activity {
                    duration: Duration::minutes(remaining_mins),
                    kind: ActivityKind::Work,
                });
            }
            Ok(Schedule {
                timetable: Timetable { entries },
                start_time: now_time.clone(),
                end_time: end_time.clone(),
                remaining: Duration::minutes(remaining_mins),
            })
        }
    }
}

fn get_duration_until(
    now_time: &DateTime<chrono_tz::Tz>,
    target: &Target,
) -> Result<(Duration, DateTime<chrono_tz::Tz>), ScheduleError> {
    // the end time is just the current yr/month/day but with a specific time
    let end_time = Utc
        .ymd(now_time.year(), now_time.month(), now_time.day())
        .with_timezone(&London)
        .and_hms_opt(target.hour, target.min, target.sec)
        .ok_or(ScheduleError::InvalidHourMinSec)?;

    match end_time.cmp(now_time) {
        Ordering::Greater => {
            let diff: Duration = end_time - *now_time;
            Ok((diff, end_time))
        }
        _ => Err(ScheduleError::NotInTheFuture),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ActivityKind::{Rest, Work};

    fn for_time_and_target(
        hr: u32,
        min: u32,
        target: Target,
    ) -> Result<Vec<(ActivityKind, i64, String)>, ScheduleError> {
        let date = Utc.ymd(2022, 8, 10);
        let nine_am = date
            .and_hms(hr - 1, min, 0)
            .with_timezone(&chrono_tz::Europe::London);
        let schedule = create_schedule(&nine_am, &target)?;
        let mut elapsed = 0;
        let mut running_time = schedule.start_time;
        let mut as_list: Vec<(ActivityKind, i64, String)> = vec![];
        for x in &schedule.timetable.entries {
            as_list.push((
                x.kind.clone(),
                x.duration.num_minutes(),
                format!("{}:{:02}", running_time.hour(), running_time.minute()),
            ));
            let curr_time = running_time.checked_add_signed(x.duration);
            if let Some(curr_time) = curr_time {
                running_time = curr_time;
            }
        }
        Ok(as_list)
    }

    #[test]
    fn test_schedule() -> Result<(), ScheduleError> {
        let schedule_entries = for_time_and_target(9, 0, Target::hour_min(11, 30))?;
        let expected = vec![
            (Work, 25, "9:00"),
            (Rest, 5, "9:25"),
            (Work, 25, "9:30"),
            (Rest, 5, "9:55"),
            (Work, 25, "10:00"),
            (Rest, 5, "10:25"),
            (Work, 25, "10:30"),
            (Rest, 5, "10:55"),
            (Work, 25, "11:00"),
            (Rest, 5, "11:25"),
        ];
        assert_eq!(schedule_entries.len(), expected.len());
        for (a, b) in schedule_entries.iter().zip(expected.iter()) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
            assert_eq!(a.2, b.2);
        }
        Ok(())
    }
}
