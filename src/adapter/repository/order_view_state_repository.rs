use fmodel_rust::materialized_view::ViewStateRepository;
use fmodel_rust::Identifier;

use crate::adapter::database::entity::OrderEntity;
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{get_all_orders, get_order, upsert_order};
use crate::application::api::OrderQueryHandler;
use crate::domain::api::OrderEvent;
use crate::domain::order_view::OrderViewState;
use crate::Database;

/// OrderViewStateRepository struct
pub struct OrderViewStateRepository {
    database: Database,
}

/// OrderViewStateRepository - struct implementation
impl OrderViewStateRepository {
    /// Create a new OrderViewStateRepository
    pub fn new(database: Database) -> Self {
        OrderViewStateRepository { database }
    }
}

/// Implementation of OrderQueryHandler for OrderViewStateRepository
impl OrderQueryHandler for OrderViewStateRepository {
    /// Get the Order view state by `id`
    async fn get_order(&self, id: &str) -> Result<Option<OrderViewState>, ErrorMessage> {
        get_order(&id.to_string(), &self.database)
            .await?
            .map(|entity| entity.to_order())
            .transpose()
    }
    /// Get all the Order view states
    async fn get_all_orders(&self) -> Result<Vec<OrderViewState>, ErrorMessage> {
        get_all_orders(&self.database)
            .await?
            .into_iter()
            .map(|entity| entity.to_order())
            .collect()
    }
}

/// OrderViewStateRepository - implementation of Fmodel ViewStateRepository for OrderEvent, OrderViewState, ErrorMessage
impl ViewStateRepository<OrderEvent, Option<OrderViewState>, ErrorMessage>
    for OrderViewStateRepository
{
    async fn fetch_state(
        &self,
        event: &OrderEvent,
    ) -> Result<Option<Option<OrderViewState>>, ErrorMessage> {
        get_order(&event.identifier(), &self.database)
            .await?
            .map(|entity| entity.to_order())
            .transpose()
            .map(Some)
    }

    async fn save(
        &self,
        state: &Option<OrderViewState>,
    ) -> Result<Option<OrderViewState>, ErrorMessage> {
        match state {
            Some(state) => {
                let order_entity = state.to_order_entity()?;
                let stored_state = upsert_order(&order_entity, &self.database).await?;
                Ok(Some(stored_state.to_order()?))
            }
            None => Ok(None),
        }
    }
}

/// Map to OrderViewState
trait ToOrder {
    fn to_order(&self) -> Result<OrderViewState, ErrorMessage>;
}

/// Map the OrderEntity to OrderViewState
impl ToOrder for OrderEntity {
    /// Map the OrderEntity to OrderViewState
    fn to_order(&self) -> Result<OrderViewState, ErrorMessage> {
        let value = self.data.clone();
        serde_json::from_value(value).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })
    }
}

/// Map to OrderEntity
trait ToOrderEntity {
    fn to_order_entity(&self) -> Result<OrderEntity, ErrorMessage>;
}
/// Map theOrderViewState to OrderEntity
impl ToOrderEntity for OrderViewState {
    /// Map theOrderViewState to OrderEntity
    fn to_order_entity(&self) -> Result<OrderEntity, ErrorMessage> {
        serde_json::to_value(self)
            .map_err(|err| ErrorMessage {
                message: err.to_string(),
            })
            .map(|value| OrderEntity {
                id: self.identifier.to_string(),
                data: value,
            })
    }
}
