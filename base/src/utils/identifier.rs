use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

/// [`TaskIdentifier`] trait used for defining unique identifiers. For example UUID, integers, strings,
/// and generally any kind of identifier format the user can use which suits their needs.
///
/// The identifier is used internally in "[`Scheduler`] Land", via a hashmap, it associates an identifier
/// with an owned [Task](crate::task::Task) instance. This identifier is unique, cloneable and comparable.
///
/// Specifically it is used in the [`SchedulerTaskStore`](crate::scheduler::task_store::SchedulerTaskStore) internally.
/// Identifiers can be configured via the [`SchedulerConfig`](crate::scheduler::SchedulerConfig) trait.
/// Different [`Schedulers`](crate::scheduler::Scheduler) may have different [`TaskIdentifiers`](TaskIdentifier)
/// defined via their configuration.
///
/// > **Note:** It should be mentioned, identifiers are held internally in some cases in the "Task Land",
/// > but never exposed directly (as to prevent leaking abstractions)
///
/// # Semantics
/// Implementors must provide a way to generate unique identifier for task via the
/// [`generate`](TaskIdentifier::generate) method as listed in the trait itself.
///
/// # Required Subtrait(s)
/// [`TaskIdentifier`] requires the following subtraits in order to be implemented:
/// - ``Debug`` - For displaying the ID.
/// - ``Clone`` - For fully cloning the ID.
/// - ``PartialEq`` - For comparing 2 IDs and checking if they are equal.
/// - ``Eq`` - For ensuring the comparison applies in both directions.
/// - ``Hash`` - For producing a hash from the ID.
///
/// [`TaskIdentifier`] also requires `Send` + `Sync` + `'static`.
///
/// # Required Method(s)
/// The [`TaskIdentifier`] trait requires developers to implement the [`generate`](TaskIdentifier::generate)
/// method, which produces a new unique identifier per call.
///
/// # Implementation(s)
/// The main implementor inside the core is [`Uuid`] which generates a random UUID v4 via using
/// internally [`Uuid::new_v4`].
///
/// # Object Safety / Dynamic Dispatching
/// This trait is **NOT** object-safe due to the `Clone` and more specifically the `Sized` supertrait requirement.
///
/// # Example(s)
/// ```
/// use uuid::Uuid;
/// use chronographer::utils::TaskIdentifier;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TaskId(Uuid);
/// impl TaskIdentifier for TaskId {
///     fn generate() -> Self {
///         TaskId(Uuid::new_v4())
///     }
/// }
///
/// let task_id1 = TaskId::generate();
/// let task_id2 = TaskId::generate();
///
/// // Unequal, as they are unique entries
/// assert_ne!(task_id1, task_id2);
///
/// fn calculate_hash<T: Hash>(t: &T) -> u64 {
///     let mut s = DefaultHasher::new();
///     t.hash(&mut s);
///     s.finish()
/// }
///
/// // They produce different hashes (since they are unique)
/// assert_ne!(calculate_hash(&task_id1), calculate_hash(&task_id2));
/// ```
/// In the example, ``TaskId`` is our identifier format (with a couple of traits implemented on top),
/// for demonstration purposes we used ``Uuid`` but as mentioned, any form of data can be used.
///
/// We implement the ``TaskIdentifier`` trait with its ``generate`` method, then we simply generate two
/// instances with that method, more specifically ``task_id1`` and ``task_id2``.
///
/// We compare the two and see they aren't equal (since they are unique), we take the hash of the two
/// and also see they are non-equal (again confirms the fact they are different).
///
/// # See Also
/// - [`Uuid`] - The default implementation, generating random v4 UUIDs.
/// - [SchedulerConfig](crate::scheduler::SchedulerConfig) - One of configuration parameters over lots of others.
/// - [Scheduler](crate::scheduler::Scheduler) - The interface around the store using the identifier.
/// - [SchedulerTaskStore](crate::scheduler::task_store::SchedulerTaskStore) - Manages linking identifiers to tasks.
/// - [`Task`](crate::task::Task) - The object which the task identifier associates.
pub trait TaskIdentifier:
'static + Debug + Clone + Eq + PartialEq<Self> + Hash + Send + Sync
{
    fn generate() -> Self;
    fn as_usize(&self) -> usize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as usize
    }
}

#[cfg(feature = "uuid")]
impl TaskIdentifier for Uuid {
    fn generate() -> Self {
        Uuid::new_v4()
    }
}

static PREV_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SnowflakeID(u64);

impl SnowflakeID {
    const CHRONOGRAPHER_EPOCH_MS: u64 = 1772985384873;
}

impl Hash for SnowflakeID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0)
    }
}

impl TaskIdentifier for SnowflakeID {
    fn generate() -> Self {
        loop {
            let current = PREV_ID.load(Ordering::Relaxed);

            let last_timestamp = current >> 16;
            let last_sequence = current & 0xFFFF;

            let now = (
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            ) - Self::CHRONOGRAPHER_EPOCH_MS;

            let (timestamp, sequence) = if now == last_timestamp {
                let next_seq = (last_sequence + 1) & 0xFFFF;

                if next_seq == 0 {
                    continue;
                }

                (now, next_seq)
            } else {
                (now, 0)
            };

            let new_id = (timestamp << 16) | sequence;

            if PREV_ID.compare_exchange(
                    current,
                    new_id,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return SnowflakeID(new_id);
            }
        }
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

