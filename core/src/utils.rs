use crate::errors::ChronographerErrors;
use crate::persistent_object::{AsPersistent, PersistenceCapability, PersistentObject};
use crate::serialized_component::SerializedComponent;
use crate::task::{TaskError, TaskHook, TaskHookContainer, TaskHookEvent};
use chrono::{DateTime, Local, TimeZone};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Map;
use std::any::{TypeId, type_name, type_name_of_val};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinSet;

pub struct PersistenceUtils(());

impl PersistenceUtils {
    pub fn serialize_field(val: impl Serialize) -> Result<serde_json::Value, TaskError> {
        serde_json::to_value(val).map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)
    }

    pub async fn serialize_persistent(
        val: &impl PersistentObject,
    ) -> Result<serde_json::Value, TaskError> {
        serde_json::to_value(val.persist().await?)
            .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)
    }

    pub async fn serialize_potential_field(
        val: &(impl AsPersistent + Send + Sync + ?Sized),
    ) -> Result<serde_json::Value, TaskError> {
        serde_json::to_value(
            match val.as_persistent().await {
                PersistenceCapability::Persistable(res) => res,
                _ => {
                    return Err(Arc::new(ChronographerErrors::NonPersistentObject(
                        type_name_of_val(&val).to_string(),
                    )));
                }
            }
            .persist()
            .await?,
        )
        .map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)
    }

    pub fn transform_serialized_to_map(
        component: SerializedComponent,
    ) -> Result<Map<String, serde_json::Value>, TaskError> {
        let repr = component.into_ir();

        match repr {
            serde_json::Value::Object(map) => Ok(map),
            other => Err(Arc::new(ChronographerErrors::NonObjectDeserialization(
                type_name_of_val(&other).to_string(),
                other,
            )) as Arc<dyn Debug + Send + Sync>),
        }
    }

    pub fn create_retrieval_error<T: ?Sized>(
        map: &Map<String, serde_json::Value>,
        error_msg: &'_ str,
    ) -> TaskError {
        Arc::new(ChronographerErrors::RetrievalFailed(
            type_name::<T>().to_string(),
            error_msg.to_string(),
            map.clone(),
        )) as Arc<dyn Debug + Send + Sync>
    }

    pub fn deserialize_partially_field<T: ?Sized>(
        map: &mut Map<String, serde_json::Value>,
        key: &'_ str,
        on_retrieve_failed_msg: &'_ str,
    ) -> Result<serde_json::Value, TaskError> {
        if map.contains_key(key) {
            return Err(Self::create_retrieval_error::<T>(
                &map,
                on_retrieve_failed_msg,
            ));
        }

        Ok(map.remove(key).unwrap())
    }

    pub fn deserialize_atomic<T: DeserializeOwned + ?Sized>(
        map: &mut Map<String, serde_json::Value>,
        key: &'_ str,
        on_retrieve_failed_msg: &'_ str,
    ) -> Result<T, TaskError> {
        let val = Self::deserialize_partially_field::<T>(map, key, on_retrieve_failed_msg)?;
        serde_json::from_value::<T>(val).map_err(|x| Arc::new(x) as Arc<dyn Debug + Send + Sync>)
    }

    pub async fn deserialize_concrete<T: PersistentObject>(
        map: &mut Map<String, serde_json::Value>,
        key: &'_ str,
        on_retrieve_failed_msg: &'_ str,
    ) -> Result<T, TaskError> {
        let frame = Self::deserialize_partially_field::<T>(map, key, on_retrieve_failed_msg)?;
        T::retrieve(
            serde_json::from_value::<SerializedComponent>(frame)
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync>)?,
        )
        .await
    }

    pub async fn deserialize_dyn<T: ?Sized, Fut: Future<Output = Result<Arc<T>, TaskError>>>(
        map: &mut Map<String, serde_json::Value>,
        key: &'_ str,
        retrieve_func: fn(SerializedComponent) -> Fut,
        on_retrieve_failed_msg: &'_ str,
    ) -> Result<Arc<T>, TaskError> {
        let val = Self::deserialize_partially_field::<T>(map, key, on_retrieve_failed_msg)?;
        retrieve_func(
            serde_json::from_value::<SerializedComponent>(val)
                .map_err(|err| Arc::new(err) as Arc<dyn Debug + Send + Sync + 'static>)?,
        )
        .await
    }
}

#[macro_export]
macro_rules! define_event {
    ($(#[$($attrss:tt)*])* $name: ident, $payload: ty) => {
        #[doc =
            concat!(
                "[`", stringify!($name), "`] is an implementation of [`TaskHookEvent`] (a system used \
                closely with [`TaskHook`]). The concrete payload type of [`", stringify!($name),
                "`] is ``", stringify!($payload), "``"
            )
        ]
        ///
        /// # Constructor(s)
        #[doc =
            concat!(
                "When constructing a [`", stringify!($name), "`] due to the fact this is a marker ``struct``, \
                making it as such zero-sized, one can either use [`", stringify!($name), "::default`]
                or via simply pasting the struct name ([`", stringify!($name), "`])"
            )
        ]
        ///
        /// # Trait Implementation(s)
        #[doc =
            concat!(
                "It is obvious that [`", stringify!($name), "`] implements the [`TaskHookEvent`], but also many \
                other traits such as [`Default`], [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], [`Eq`] \
                and [`Hash`] from the standard Rust side, as well as [`Serialize`] and [`Deserialize`]"
            )
        ]
        ///
        /// # Cloning Semantics
        #[doc = concat!(
            "When cloning / copy a [`", stringify!($name), "`] it fully creates a \
            new independent version of that instance"
        )]
        ///
        $(#[$($attrss)*])*
        /// - [`TaskHook`]
        /// - [`TaskHookEvent`]
        /// - [`Task`]
        /// - [`TaskFrame`]
        #[derive(Default, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a> TaskHookEvent for $name {
            type Payload = $payload;
            const PERSISTENCE_ID: &'static str = concat!("chronographer_core#", stringify!($name));
        }
    };
}

/// Simply converts the ``SystemTime`` to a ``DateTime<Local>``, it is a private
/// method used internally by ChronoGrapher, as such why it lives in utils module
pub(crate) fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
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
pub(crate) fn date_time_to_system_time(dt: DateTime<impl TimeZone>) -> SystemTime {
    let duration_since_epoch = dt.timestamp_nanos_opt().unwrap();
    if duration_since_epoch >= 0 {
        UNIX_EPOCH + Duration::from_nanos(duration_since_epoch as u64)
    } else {
        UNIX_EPOCH - Duration::from_nanos((-duration_since_epoch) as u64)
    }
}

pub(crate) async fn emit_event<E: TaskHookEvent>(
    hooks_container: &TaskHookContainer,
    payload: &E::Payload,
) {
    let hooks = hooks_container
        .0
        .get(&TypeId::of::<E>())
        .map(|x| x.value())
        .unwrap_or_default();

    let mut set = JoinSet::new();

    for hook in hooks {
        set.spawn(hook.on_emit(&payload));
    }

    set.join_all().await;
}
