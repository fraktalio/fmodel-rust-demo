use crate::adapter::repository::ToNewEventEntity;
use fmodel_rust::aggregate::EventRepository;
use uuid::Uuid;

use crate::adapter::database::entity::{EventEntity, NewEventEntity};
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{append_event, list_events};
use crate::domain::api::{Command, Event, Identifier, OrderEvent, RestaurantEvent};
use crate::Database;

/// EventRepository struct
pub struct AggregateEventRepository {
    database: Database,
}

impl AggregateEventRepository {
    /// Create a new EventRepository
    pub fn new(database: Database) -> Self {
        AggregateEventRepository { database }
    }
}

/// EventRepository - implementation of Fmodel EventRepository for Command, Event, Uuid, ErrorMessage
impl EventRepository<Command, Event, Uuid, ErrorMessage> for AggregateEventRepository {
    async fn fetch_events(&self, command: &Command) -> Result<Vec<(Event, Uuid)>, ErrorMessage> {
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

    async fn save(
        &self,
        events: &[Event],
        latest_version: &Option<Uuid>,
    ) -> Result<Vec<(Event, Uuid)>, ErrorMessage> {
        let mut latest_version = latest_version.to_owned();
        let mut result = Vec::new();

        for event in events {
            let event_request = event.to_new_event_entity(latest_version)?;
            append_event(&event_request, &self.database).await?;
            latest_version = Some(event_request.event_id);
            result.push(((*event).to_owned(), event_request.event_id));
        }

        Ok(result)
    }
}

/// Map the EventEntity into the domain events
pub trait ToEvent {
    fn to_event(&self) -> Result<Event, ErrorMessage>;
}

/// Map the EventEntity into the domain events
impl ToEvent for EventEntity {
    fn to_event(&self) -> Result<Event, ErrorMessage> {
        let value = self.data.clone();
        serde_json::from_value(value).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })
    }
}

/// Map from domain events of type OrderEvent to EventEntity
impl ToNewEventEntity for Event {
    fn to_new_event_entity(&self, version: Option<Uuid>) -> Result<NewEventEntity, ErrorMessage> {
        let data = serde_json::to_value(self).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })?;
        Ok(match self {
            Event::Second(event) => match event {
                OrderEvent::Created(event) => NewEventEntity {
                    event: "OrderCreated".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Order".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                OrderEvent::Prepared(event) => NewEventEntity {
                    event: "OrderPrepared".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Order".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                OrderEvent::NotCreated(event) => NewEventEntity {
                    event: "OrderNotCreated".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Order".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                OrderEvent::NotPrepared(event) => NewEventEntity {
                    event: "OrderNotPrepared".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Order".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
            },
            Event::First(event) => match event {
                RestaurantEvent::Created(event) => NewEventEntity {
                    event: "RestaurantCreated".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                RestaurantEvent::NotCreated(event) => NewEventEntity {
                    event: "RestaurantNotCreated".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                RestaurantEvent::MenuChanged(event) => NewEventEntity {
                    event: "RestaurantMenuChanged".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                RestaurantEvent::MenuNotChanged(event) => NewEventEntity {
                    event: "RestaurantMenuNotChanged".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                RestaurantEvent::OrderPlaced(event) => NewEventEntity {
                    event: "RestaurantOrderPlaced".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
                RestaurantEvent::OrderNotPlaced(event) => NewEventEntity {
                    event: "RestaurantOrderNotPlaced".to_string(),
                    event_id: Uuid::new_v4(),
                    decider: "Restaurant".to_string(),
                    decider_id: event.identifier.0.to_string(),
                    data,
                    command_id: None,
                    previous_id: version,
                    r#final: false,
                },
            },
        })
    }
}
