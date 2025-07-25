use std::sync::Arc;

use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{ack_event, nack_event, stream_events};
use crate::adapter::repository::event_repository::ToEvent;
use crate::adapter::repository::order_view_state_repository::OrderViewStateRepository;
use crate::adapter::repository::restaurant_view_state_repository::RestaurantViewStateRepository;
use crate::application::api::{OrderMaterializedView, RestaurantMaterializedView};
use crate::Database;
use log::{debug, error, warn};

/// Stream events to the materialized view - Simple implementation
pub async fn stream_events_to_view(
    restaurant_materialized_view: Arc<
        RestaurantMaterializedView<'_, RestaurantViewStateRepository>,
    >,
    order_materialized_view: Arc<OrderMaterializedView<'_, OrderViewStateRepository>>,
    db: &Database,
) -> Result<(), ErrorMessage> {
    // Stream events from the `event` table to the materialized view of name "view"
    match stream_events(&"view".to_string(), db).await {
        Ok(Some(event_entity)) => {
            debug!("Processing Event: {event_entity:?}");
            match event_entity.decider.as_str() {
                "Restaurant" => {
                    match restaurant_materialized_view
                        .handle(&event_entity.to_event()?)
                        .await
                    {
                        Ok(_) => {
                            debug!("Restaurant materialized view updated successfully");
                            ack_event(
                                &event_entity.offset,
                                &"view".to_string(),
                                &event_entity.decider_id,
                                db,
                            )
                            .await
                            .map(drop)
                        }
                        Err(error) => {
                            error!(
                                "Restaurant materialized view update failed: {}",
                                error.message
                            );
                            nack_event(&"view".to_string(), &event_entity.decider_id, db)
                                .await
                                .map(drop)
                        }
                    }
                }
                "Order" => {
                    match order_materialized_view
                        .handle(&event_entity.to_event()?)
                        .await
                    {
                        Ok(_) => {
                            debug!("Order materialized view updated successfully");
                            ack_event(
                                &event_entity.offset,
                                &"view".to_string(),
                                &event_entity.decider_id,
                                db,
                            )
                            .await
                            .map(drop)
                        }
                        Err(error) => {
                            error!("Order materialized view update failed: {}", error.message);
                            nack_event(&"view".to_string(), &event_entity.decider_id, db)
                                .await
                                .map(drop)
                        }
                    }
                }
                _ => {
                    warn!("Unknown event type: {}", event_entity.event);
                    ack_event(
                        &event_entity.offset,
                        &"view".to_string(),
                        &event_entity.decider_id,
                        db,
                    )
                    .await
                    .map(drop)
                }
            }
        }
        Ok(None) => {
            debug!("No events to process, continue with the next iteration");
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
