use crate::errors::ChronographerErrors;
use crate::persistent_object::PersistentObject;
use crate::schedule::TaskSchedule;
use crate::scheduling_strats::ScheduleStrategy;
use crate::serialized_component::SerializedComponent;
use crate::task::conditionframe::ConditionalFramePredicate;
use crate::task::dependency::FrameDependency;
use crate::task::dependencyframe::DependentFailBehavior;
use crate::task::selectframe::SelectFrameAccessor;
use crate::task::{
    MetadataEventListener, ObserverFieldListener, TaskError, TaskErrorHandler, TaskEventListener,
    TaskFrame,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::pin::Pin;
use std::sync::Arc;

pub static RETRIEVE_REGISTRIES: Lazy<RetrieveRegistries> =
    Lazy::new(|| RetrieveRegistries::default());

pub type RetrievedFut<T> = Pin<Box<dyn Future<Output = Result<Arc<T>, TaskError>> + Send>>;
pub type RetrieveFunc<T> = fn(SerializedComponent) -> RetrievedFut<T>;
pub type RetrieveRegisters<T> = DashMap<&'static str, RetrieveFunc<T>>;

#[derive(Default)]
pub struct RetrieveRegistries {
    conditional_predicate_registries: RetrieveRegisters<dyn ConditionalFramePredicate>,
    select_frame_accessors_registries: RetrieveRegisters<dyn SelectFrameAccessor>,
    task_frame_registries: RetrieveRegisters<dyn TaskFrame>,
    frame_dependency_registries: RetrieveRegisters<dyn FrameDependency>,
    task_schedule_registries: RetrieveRegisters<dyn TaskSchedule>,
    task_error_handler_registries: RetrieveRegisters<dyn TaskErrorHandler>,
    task_schedule_strategy_registries: RetrieveRegisters<dyn ScheduleStrategy>,
    metadata_event_listener_registries: RetrieveRegisters<dyn MetadataEventListener>,
    /*
       TODO: Find a way to store arbitrary TaskEventListener and ObserverFieldListener,
       TODO: and find some way to deserialize them while retaining their original payloads
    */
    task_event_listener_registries: RetrieveRegisters<dyn TaskEventListener<P>>,
    observer_event_listener_registries: RetrieveRegisters<dyn ObserverFieldListener<T>>,
    frame_dependent_fail_behaviour_registries: RetrieveRegisters<dyn DependentFailBehavior>,
}

macro_rules! implement_registries_for {
    ($register_method: ident, $retrieve_method: ident, $target_type: tt, $target_registries: ident) => {
        impl RetrieveRegistries {
            #[doc = "Executes the corresponding retrieve method based on a serialized component ID"]
            #[doc =
                concat!(
                    "via the registry system. This is one of many methods to \
                    use when retrieving a specific type, it should be used when one knows \
                    that their type will be [`",
                    stringify!($target_type),
                    "`]"
                )
            ]
            #[doc = ""]
            #[doc = " # Argument(s)"]
            #[doc = " This method accepts one argument, that being the [`SerializedComponent`] as"]
            #[doc = " ``component`` which contains the JSON data and the ID, forming an Intermediate"]
            #[doc = " Representation (IR)"]
            #[doc = " "]
            #[doc = " # Returns"]
            #[doc = concat!(
                "A result of the deserialized [`", stringify!($target_type), "`] if successful, \
                otherwise an error indicating where it could possibly have failed"
            )]
            #[doc = " "]
            #[doc = " # See Also"]
            #[doc = concat!("- [`SerializedComponent`]")]
            #[doc = concat!("- [`", stringify!($target_type), "`]")]
            #[doc = concat!("- [`RetrieveRegistries`]")]
            pub async fn $retrieve_method(
                &self,
                component: SerializedComponent
            ) -> Result<Arc<dyn $target_type>, TaskError> {
                if let Some(retriever) = self.$target_registries.get(component.id()) {
                    let val = retriever.value()(component).await?;
                    return Ok(val)
                }
                Err(Arc::new(ChronographerErrors::NonMatchingIDs(component.id().to_string())))
            }

            #[doc = "Registers a corresponding retrieval method (given the type implements PersistentObject"]
            #[doc =
                concat!(
                    " as well as the [`", stringify!($target_type), "`] trait). Registering it, makes \
                    it possible to use in dynamic context where the object's type is not known directly \
                    but rather via an ID stored in a [`SerializedComponent`]"
                )
            ]
            #[doc = " # See Also"]
            #[doc = concat!("- [`SerializedComponent`]")]
            #[doc = concat!("- [`", stringify!($target_type), "`]")]
            #[doc = concat!("- [`RetrieveRegistries`]")]
            pub fn $register_method<T: $target_type + PersistentObject + 'static>(&self) {
                fn retrieve_wrapper<T: $target_type + PersistentObject + 'static>(
                    component: SerializedComponent,
                ) -> RetrievedFut<dyn $target_type>
                {
                    Box::pin(async move {
                        let concrete = T::retrieve(component).await?;
                        Ok(Arc::new(concrete) as Arc<dyn $target_type>)
                    })
                }

                self.$target_registries
                    .insert(T::persistence_id(), retrieve_wrapper::<T>);
            }
        }
    };
}

implement_registries_for!(
    register_conditional_predicate,
    retrieve_conditional_predicate,
    ConditionalFramePredicate,
    conditional_predicate_registries
);

implement_registries_for!(
    register_task_frame,
    retrieve_task_frame,
    TaskFrame,
    task_frame_registries
);

implement_registries_for!(
    register_frame_dependency,
    retrieve_frame_dependency,
    FrameDependency,
    frame_dependency_registries
);

implement_registries_for!(
    register_task_schedule_dependency,
    retrieve_task_schedule_dependency,
    TaskSchedule,
    task_schedule_registries
);

implement_registries_for!(
    register_task_error_handler,
    retrieve_task_error_handler,
    TaskErrorHandler,
    task_error_handler_registries
);

implement_registries_for!(
    register_task_schedule_strategy,
    retrieve_task_schedule_strategy,
    ScheduleStrategy,
    task_schedule_strategy_registries
);

implement_registries_for!(
    register_metadata_listener,
    retrieve_metadata_listener,
    MetadataEventListener,
    metadata_event_listener_registries
);

implement_registries_for!(
    register_dependent_fail_behaviour,
    retrieve_dependent_fail_behaviour,
    DependentFailBehavior,
    frame_dependent_fail_behaviour_registries
);

implement_registries_for!(
    register_select_frame_accessor,
    retrieve_select_frame_accessor,
    SelectFrameAccessor,
    select_frame_accessors_registries
);
