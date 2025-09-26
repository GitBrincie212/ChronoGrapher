use chrono::{DateTime, Local, TimeZone};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    Local.timestamp_opt(sec, nsec).unwrap()
}

pub fn date_time_to_system_time(dt: DateTime<impl TimeZone>) -> SystemTime {
    let duration_since_epoch = dt.timestamp_nanos_opt().unwrap();
    if duration_since_epoch >= 0 {
        UNIX_EPOCH + Duration::from_nanos(duration_since_epoch as u64)
    } else {
        UNIX_EPOCH - Duration::from_nanos((-duration_since_epoch) as u64)
    }
}
