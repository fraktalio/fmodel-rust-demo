use fmodel_rust::aggregate::EventRepository;
use log::{info, warn};
use uuid::Uuid;

use crate::adapter::database::entity::{EventEntity, NewEventEntity};
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{append_event, get_latest_event, list_events};
use crate::domain::api::{DeciderName, EventName, Identifier};
use crate::{adapter, Database};
/// EventRepository struct
pub struct AggregateEventRepository {
    database: Database,
}

// General Event repository
impl AggregateEventRepository {
    /// Create a new EventRepository
    pub fn new(database: Database) -> Self {
        AggregateEventRepository { database }
    }
}

/// EventRepository - implementation of Fmodel EventRepository for C, E, Uuid, ErrorMessage, where C and E are constrained with specific traits
impl<C, E> EventRepository<C, E, Uuid, ErrorMessage> for AggregateEventRepository
where
    C: Identifier + Sync,
    E: Identifier
        + std::fmt::Debug
        + Sync
        + Send
        + serde::de::DeserializeOwned
        + Clone
        + ToEventEntity,
{
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Uuid)>, ErrorMessage> {
        // https://doc.rust-lang.org/rust-by-example/error/iter_result.html#fail-the-entire-operation-with-collect
        list_events(&command.identifier(), &self.database)
            .await?
            .into_iter()
            .map(|event_entity| {
                event_entity
                    .to_event()
                    .map(|event| (event, event_entity.event_id))
            })
            .collect()
    }

    async fn save(&self, events: &[E]) -> Result<Vec<(E, Uuid)>, ErrorMessage> {
        let first_event: &E = events.first().unwrap();
        let mut latest_version: Option<Uuid> = <adapter::repository::event_repository::AggregateEventRepository as fmodel_rust::aggregate::EventRepository<C, E, uuid::Uuid, adapter::database::error::ErrorMessage>>::version_provider(self,first_event).await?;
        //let mut latest_version: Option<Uuid> = self.version_provider(first_event).await?;
        let mut result = Vec::new();

        for event in events {
            let event_request = event.to_event_entity(latest_version)?;
            append_event(&event_request, &self.database).await?;
            latest_version = Some(event_request.event_id);
            result.push(((*event).to_owned(), event_request.event_id));
        }

        Ok(result)
    }
    async fn version_provider(&self, event: &E) -> Result<Option<Uuid>, ErrorMessage> {
        get_latest_event(&event.identifier(), &self.database)
            .await
            .map(|event_entity| event_entity.map(|e| e.event_id))
    }
}

/// Map the EventEntity into the domain events
pub trait ToEvent<E> {
    fn to_event(&self) -> Result<E, ErrorMessage>;
}

/// Map the EventEntity into the domain events
impl<E> ToEvent<E> for EventEntity
where
    E: serde::de::DeserializeOwned,
{
    fn to_event(&self) -> Result<E, ErrorMessage> {
        let value = self.data.clone();
        serde_json::from_value(value).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })
    }
}

trait ToEventEntity {
    fn to_event_entity(&self, version: Option<Uuid>) -> Result<NewEventEntity, ErrorMessage>;
}
/// Map from domain events of type OrderEvent to EventEntity
impl<E> ToEventEntity for E
where
    E: Identifier + EventName + DeciderName + serde::ser::Serialize,
{
    fn to_event_entity(&self, version: Option<Uuid>) -> Result<NewEventEntity, ErrorMessage> {
        let data = serde_json::to_value(self).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })?;

        Ok(NewEventEntity {
            event: self.event_name(),
            event_id: Uuid::new_v4(),
            decider: self.decider_name(),
            decider_id: self.identifier(),
            data,
            command_id: None,
            previous_id: version,
            r#final: false,
        })
    }
}
