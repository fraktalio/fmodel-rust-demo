use async_trait::async_trait;

use fmodel_rust::materialized_view::ViewStateRepository;

use crate::adapter::database::entity::RestaurantEntity;
use crate::adapter::database::error::ErrorMessage;
use crate::adapter::database::queries::{get_all_restaurants, get_restaurant, upsert_restaurant};
use crate::application::api::RestaurantQueryHandler;
use crate::domain::api::{Identifier, RestaurantEvent};
use crate::domain::restaurant_view::RestaurantViewState;
use crate::Database;

/// RestaurantViewStateRepository struct
pub struct RestaurantViewStateRepository {
    database: Database,
}

/// RestaurantViewStateRepository - struct implementation
impl RestaurantViewStateRepository {
    /// Create a new RestaurantViewStateRepository
    pub fn new(database: Database) -> Self {
        RestaurantViewStateRepository { database }
    }
}

/// Implementation of RestaurantQueryHandler for RestaurantViewStateRepository
#[async_trait]
impl RestaurantQueryHandler for RestaurantViewStateRepository {
    /// Get the Restaurant view state by `id`
    async fn get_restaurant(&self, id: &str) -> Result<Option<RestaurantViewState>, ErrorMessage> {
        get_restaurant(&id.to_string(), &self.database)
            .await?
            .map(|entity| entity.to_restaurant())
            .transpose()
    }
    /// Get all the Restaurant view states
    async fn get_all_restaurants(&self) -> Result<Vec<RestaurantViewState>, ErrorMessage> {
        get_all_restaurants(&self.database)
            .await?
            .into_iter()
            .map(|entity| entity.to_restaurant())
            .collect()
    }
}

/// RestaurantViewStateRepository - implementation of Fmodel ViewStateRepository for RestaurantEvent, RestaurantViewState, ErrorMessage
#[async_trait]
impl ViewStateRepository<RestaurantEvent, Option<RestaurantViewState>, ErrorMessage>
    for RestaurantViewStateRepository
{
    async fn fetch_state(
        &self,
        event: &RestaurantEvent,
    ) -> Result<Option<Option<RestaurantViewState>>, ErrorMessage> {
        get_restaurant(&event.identifier(), &self.database)
            .await?
            .map(|entity| entity.to_restaurant())
            .transpose()
            .map(Some)
    }

    async fn save(
        &self,
        state: &Option<RestaurantViewState>,
    ) -> Result<Option<RestaurantViewState>, ErrorMessage> {
        match state {
            Some(state) => {
                let restaurant_entity = state.to_restaurant_entity()?;
                let stored_state = upsert_restaurant(&restaurant_entity, &self.database).await?;
                Ok(Some(stored_state.to_restaurant()?))
            }
            None => Ok(None),
        }
    }
}

/// Map to RestaurantViewState
trait ToRestaurant {
    fn to_restaurant(&self) -> Result<RestaurantViewState, ErrorMessage>;
}

/// Map the RestaurantEntity to RestaurantViewState
impl ToRestaurant for RestaurantEntity {
    /// Map the RestaurantEntity to RestaurantViewState
    fn to_restaurant(&self) -> Result<RestaurantViewState, ErrorMessage> {
        let value = self.data.clone();
        serde_json::from_value(value).map_err(|err| ErrorMessage {
            message: err.to_string(),
        })
    }
}

/// Map to RestaurantEntity
trait ToRestaurantEntity {
    fn to_restaurant_entity(&self) -> Result<RestaurantEntity, ErrorMessage>;
}
/// Map RestaurantViewState to RestaurantEntity
impl ToRestaurantEntity for RestaurantViewState {
    /// Map the RestaurantViewState to RestaurantEntity
    fn to_restaurant_entity(&self) -> Result<RestaurantEntity, ErrorMessage> {
        serde_json::to_value(self)
            .map_err(|err| ErrorMessage {
                message: err.to_string(),
            })
            .map(|value| RestaurantEntity {
                id: self.identifier.to_string(),
                data: value,
            })
    }
}
