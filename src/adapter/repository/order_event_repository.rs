use crate::adapter::repository::ToNewEventEntity;
use async_trait::async_trait;
use fmodel_rust::aggregate::EventRepository;
use uuid::Uuid;

use crate::adapter::database::entity::{EventEntity, NewEventEntity};
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{append_event, list_events};
use crate::domain::api::{OrderCommand, OrderEvent};
use crate::Database;

/// OrderEventRepository struct
pub struct OrderEventRepository {
    database: Database,
}

impl OrderEventRepository {
    /// Create a new OrderEventRepository
    pub fn new(database: Database) -> Self {
        OrderEventRepository { database }
    }
}

/// OrderEventRepository - implementation of Fmodel EventRepository for OrderCommand, OrderEvent, Uuid, ErrorMessage
#[async_trait]
impl EventRepository<OrderCommand, OrderEvent, Uuid, ErrorMessage> for OrderEventRepository {
    async fn fetch_events(
        &self,
        command: &OrderCommand,
    ) -> Result<Vec<(OrderEvent, Uuid)>, ErrorMessage> {
        let command_id = match command {
            OrderCommand::Create(cmd) => cmd.identifier.to_string(),
            OrderCommand::MarkAsPrepared(cmd) => cmd.identifier.to_string(),
        };

        // https://doc.rust-lang.org/rust-by-example/error/iter_result.html#fail-the-entire-operation-with-collect
        list_events(&command_id, &self.database)
            .await?
            .into_iter()
            .map(|event_entity| {
                event_entity
                    .to_order_event()
                    .map(|event| (event, event_entity.event_id))
            })
            .collect()
    }

    async fn save(
        &self,
        events: &[OrderEvent],
        latest_version: &Option<Uuid>,
    ) -> Result<Vec<(OrderEvent, Uuid)>, ErrorMessage> {
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

/// Map the EventEntity into the Order domain events
pub trait ToOrderEvent {
    fn to_order_event(&self) -> Result<OrderEvent, ErrorMessage>;
}

/// Map the EventEntity into the Order domain events
impl ToOrderEvent for EventEntity {
    fn to_order_event(&self) -> Result<OrderEvent, ErrorMessage> {
        let value = self.data.clone();
        serde_json::from_value(value).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })
    }
}

/// Map from domain events of type OrderEvent to EventEntity
impl ToNewEventEntity for OrderEvent {
    fn to_new_event_entity(&self, version: Option<Uuid>) -> Result<NewEventEntity, ErrorMessage> {
        let data = serde_json::to_value(self).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })?;
        Ok(match self {
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
        })
    }
}
