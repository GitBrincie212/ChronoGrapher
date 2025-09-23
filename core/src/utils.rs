use chrono::{DateTime, Local, TimeZone};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[macro_export]
macro_rules! policy_match {
    (
        $metadata: expr, $emitter: expr, $task: expr,
        $self: expr, $result: expr, $policy_enum: ident
    ) => {{
        match ($result, &$self.policy) {
            (Err(error), &$policy_enum::RunUntilFailure) => {
                $emitter
                    .clone()
                    .emit(
                        $metadata.clone(),
                        $self.on_child_end.clone(),
                        ($task.clone(), Some(error.clone())),
                    )
                    .await;
                return Err(error);
            }
            (Ok(res), &$policy_enum::RunUntilSuccess) => {
                $emitter
                    .clone()
                    .emit(
                        $metadata.clone(),
                        $self.on_child_end.clone(),
                        ($task.clone(), None),
                    )
                    .await;
                return Ok(res);
            }
            (Err(error), _) => {
                $emitter
                    .clone()
                    .emit(
                        $metadata.clone(),
                        $self.on_child_end.clone(),
                        ($task.clone(), Some(error.clone())),
                    )
                    .await;
            }
            (Ok(_), _) => {
                $emitter
                    .clone()
                    .emit(
                        $metadata.clone(),
                        $self.on_child_end.clone(),
                        ($task.clone(), None),
                    )
                    .await;
            }
        }
    }};
}

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
