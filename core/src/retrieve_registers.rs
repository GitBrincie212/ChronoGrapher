use std::pin::Pin;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use crate::persistent_object::PersistentObject;
use crate::serialized_component::SerializedComponent;
use crate::task::conditionframe::ConditionalFramePredicate;
use crate::task::TaskError;

pub static RETRIEVE_REGISTRIES: Lazy<RetrieveRegistries> = Lazy::new(|| {
    RetrieveRegistries::new()
});

pub type RetrievedFut<T> = Pin<Box<dyn Future<Output = Result<T, TaskError>> + Send>>;
pub type RetrieveFunc<T: PersistentObject> = fn(SerializedComponent) -> RetrievedFut<T>;
pub type RetrieveRegistry<T: PersistentObject> = DashMap<&'static str, RetrieveFunc<T>>;

pub struct RetrieveRegistries {
    conditional_predicate_registries: RetrieveRegistry<Box<dyn ConditionalFramePredicate>>
}

impl RetrieveRegistries {
    fn new() -> Self {
        Self {
            conditional_predicate_registries: DashMap::new()
        }
    }

    fn register_conditional_predicate<T: ConditionalFramePredicate + PersistentObject + 'static>(&self) {
        fn retrieve_wrapper<T>(
            component: SerializedComponent,
        ) -> RetrievedFut<Box<dyn ConditionalFramePredicate>>
        where
            T: ConditionalFramePredicate + PersistentObject + 'static,
        {
            Box::pin(async move {
                let concrete = T::retrieve(component).await?;
                Ok(Box::new(concrete) as Box<dyn ConditionalFramePredicate>)
            })
        }

        self.conditional_predicate_registries
            .insert(T::persistence_id(), retrieve_wrapper::<T>);
    }
}