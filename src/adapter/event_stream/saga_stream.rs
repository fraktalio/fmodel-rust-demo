use std::sync::Arc;

use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{ack_event, nack_event, stream_events};
use crate::adapter::publisher::order_action_publisher::OrderActionPublisher;
use crate::adapter::repository::event_repository::ToEvent;
use crate::application::api::OrderSagaManager;
use crate::Database;
use log::{debug, error, warn};

/// Stream events to the saga manager - Simple implementation
pub async fn stream_events_to_saga(
    order_saga_manager: Arc<OrderSagaManager<'_, OrderActionPublisher<'_>>>,
    db: &Database,
) -> Result<(), ErrorMessage> {
    // Stream events from the `event` table to the saga manager of name "saga"
    // NOTE: Saga manager is also an event handler
    match stream_events(&"saga".to_string(), db).await {
        Ok(Some(event_entity)) => {
            debug!("Processing Event in Saga: {:?}", event_entity);
            match event_entity.decider.as_str() {
                "Restaurant" => match order_saga_manager.handle(&event_entity.to_event()?).await {
                    Ok(_) => {
                        debug!("Order Saga executed successfully");
                        ack_event(
                            &event_entity.offset,
                            &"saga".to_string(),
                            &event_entity.decider_id,
                            db,
                        )
                        .await
                        .map(drop)
                    }
                    Err(error) => {
                        error!("Order Saga failed: {}", error.message);
                        nack_event(&"saga".to_string(), &event_entity.decider_id, db)
                            .await
                            .map(drop)
                    }
                },
                _ => {
                    warn!("Unknown event type: {}", event_entity.event);
                    ack_event(
                        &event_entity.offset,
                        &"saga".to_string(),
                        &event_entity.decider_id,
                        db,
                    )
                    .await
                    .map(drop)
                }
            }
        }
        Ok(None) => {
            debug!("No events to process in SAGA, continue with the next iteration");
            Ok(())
        }
        Err(error) => {
            error!("Error: {}", error.message);
            Err(ErrorMessage {
                message: error.message,
            })
        }
    }
}
