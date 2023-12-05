use crate::adapter::database::entity::NewEventEntity;
use crate::adapter::database::error::ErrorMessage;
use uuid::Uuid;

pub mod order_event_repository;
pub mod order_view_state_repository;
pub mod restaurant_event_repository;
pub mod restaurant_view_state_repository;

/// Map the domain events into the EventEntity
trait ToNewEventEntity {
    fn to_new_event_entity(&self, version: Option<Uuid>) -> Result<NewEventEntity, ErrorMessage>;
}
