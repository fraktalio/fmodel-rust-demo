use std::collections::HashMap;

use fmodel_rust::aggregate::EventRepository;
use fmodel_rust::Identifier;
use uuid::Uuid;

use crate::adapter::database::entity::{EventEntity, NewEventEntity};
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{append_events, get_latest_event, list_events};
use crate::domain::api::{DeciderName, EventName};
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
        log::debug!("Fetching events for command: {:?}", command.identifier());
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
        let mut result_events = Vec::new();
        let mut new_events = Vec::new();
        // Key is the identifier (decider_id) of the event, value is the latest version of the event for this partition/stream/decider_id
        let mut latest_versions: HashMap<String, Uuid> = HashMap::new();

        for event in events {
            let latest_version = match latest_versions.get(&event.identifier()) {
                Some(&v) => Some(v),
                None => {
                    let v = <adapter::repository::event_repository::AggregateEventRepository as fmodel_rust::aggregate::EventRepository<C, E, uuid::Uuid, adapter::database::error::ErrorMessage>>::version_provider(self,event).await?;
                    if let Some(version) = v {
                        latest_versions.insert(event.identifier().to_owned(), version);
                    }
                    v
                }
            };
            let event_request = event.to_event_entity(latest_version)?;
            result_events.push(((*event).to_owned(), event_request.event_id));
            new_events.push(event_request.to_owned());
            // Update the latest version of the event for this partition/stream/decider_id
            latest_versions.insert(event.identifier().to_owned(), event_request.event_id);
        }
        log::debug!("Saving events...");
        append_events(&new_events, &self.database).await?;
        Ok(result_events)
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
