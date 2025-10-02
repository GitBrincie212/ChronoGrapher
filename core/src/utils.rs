use chrono::{DateTime, Local, TimeZone};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[macro_export]
macro_rules! to_json {
    ($val: expr) => {{
        serde_json::to_value(
            $val
        ).map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)?
    }};
}

#[macro_export]
macro_rules! deserialize_field {
    ($repr: expr, $var_name: ident, $target: expr, $target_type: ty, $error_msg: expr) => {
        if $repr.contains_key($target) {
            return Err(deserialization_err!(
                $repr,
                $target_type,
                $error_msg
            ))
        }
        
        let $var_name = $repr.remove($target).unwrap();
    };
}

#[macro_export]
macro_rules! deserialization_err {
    ($repr: expr, $target_type: ty, $error_msg: expr) => {{
        Arc::new(ChronographerErrors::DeserializationFailed(
            stringify!($target_type).to_string(),
            $error_msg.to_string(),
            $repr.clone()
        )) as Arc<dyn Debug + Send + Sync>
    }};
}

#[macro_export]
macro_rules! acquire_mut_ir_map {
    ($target_type: ty, $component: expr) => {{
        let repr = $component.into_ir();

        match repr {
            serde_json::Value::Object(map) => map,
            other => {
                return Err(Arc::new(ChronographerErrors::NonObjectDeserialization(
                    stringify!($target_type).to_string(),
                    other
                )) as Arc<dyn Debug + Send + Sync>)
            }
        }
    }};
}


/// Simply converts the ``SystemTime`` to a ``DateTime<Local>``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
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

/// Simply converts the ``DateTime<Local>`` to a ``SystemTime``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
pub fn date_time_to_system_time(dt: DateTime<impl TimeZone>) -> SystemTime {
    let duration_since_epoch = dt.timestamp_nanos_opt().unwrap();
    if duration_since_epoch >= 0 {
        UNIX_EPOCH + Duration::from_nanos(duration_since_epoch as u64)
    } else {
        UNIX_EPOCH - Duration::from_nanos((-duration_since_epoch) as u64)
    }
}
