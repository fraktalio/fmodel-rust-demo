use async_trait::async_trait;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use fmodel_rust::decider::Decider;
use fmodel_rust::materialized_view::MaterializedView;
use fmodel_rust::saga::Saga;
use fmodel_rust::saga_manager::SagaManager;
use fmodel_rust::view::View;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::database::error::ErrorMessage;
use crate::domain::api::{OrderCommand, OrderEvent, RestaurantCommand, RestaurantEvent};
use crate::domain::order_decider::Order;
use crate::domain::order_view::OrderViewState;
use crate::domain::restaurant_decider::Restaurant;
use crate::domain::restaurant_view::RestaurantViewState;

/// Convenient OrderAggregate type alias - Command side of CQRS pattern
pub type OrderAggregate<'a, R> = EventSourcedAggregate<
    OrderCommand,
    Option<Order>,
    OrderEvent,
    R,
    Decider<'a, OrderCommand, Option<Order>, OrderEvent>,
    Uuid,
    ErrorMessage,
>;

/// Convenient RestaurantAggregate type alias - Command side of CQRS pattern
pub type RestaurantAggregate<'a, R> = EventSourcedAggregate<
    RestaurantCommand,
    Option<Restaurant>,
    RestaurantEvent,
    R,
    Decider<'a, RestaurantCommand, Option<Restaurant>, RestaurantEvent>,
    Uuid,
    ErrorMessage,
>;

/// RestaurantQueryHandler trait - Query side of CQRS pattern
#[async_trait]
pub trait RestaurantQueryHandler {
    /// Get the Restaurant view state by `id`
    async fn get_restaurant(&self, id: &str) -> Result<Option<RestaurantViewState>, ErrorMessage>;
    /// Get all the Restaurant view states
    async fn get_all_restaurants(&self) -> Result<Vec<RestaurantViewState>, ErrorMessage>;
}

/// OrderQueryHandler trait - Query side of CQRS pattern
#[async_trait]
pub trait OrderQueryHandler {
    /// Get the Order view state by `id`
    async fn get_order(&self, id: &str) -> Result<Option<OrderViewState>, ErrorMessage>;
    /// Get all the Order view states
    async fn get_all_orders(&self) -> Result<Vec<OrderViewState>, ErrorMessage>;
}

/// Application struct - A product of the application layer - A cluster of command handling (aggregate) and query handling components
pub struct Application<
    'a,
    OR: EventRepository<OrderCommand, OrderEvent, Uuid, ErrorMessage>,
    RR: EventRepository<RestaurantCommand, RestaurantEvent, Uuid, ErrorMessage>,
    OQH: OrderQueryHandler,
    RQH: RestaurantQueryHandler,
> {
    /// Restaurant aggregate - Command side of CQRS pattern - Command handler for Restaurant
    pub restaurant_aggregate: Arc<RestaurantAggregate<'a, RR>>,
    /// Order aggregate - Command side of CQRS pattern - Command handler for Order
    pub order_aggregate: Arc<OrderAggregate<'a, OR>>,
    /// Restaurant query handler - Query side of CQRS pattern - Query handler for Restaurant
    pub restaurant_query_handler: RQH,
    /// Order query handler - Query side of CQRS pattern - Query handler for Order
    pub order_query_handler: OQH,
}

/// Convenient OrderMaterializedView type alias - Query side of CQRS pattern
pub type RestaurantMaterializedView<'a, R> = MaterializedView<
    Option<RestaurantViewState>,
    RestaurantEvent,
    R,
    View<'a, Option<RestaurantViewState>, RestaurantEvent>,
    ErrorMessage,
>;
/// Convenient OrderMaterializedView type alias - Query side of CQRS pattern
pub type OrderMaterializedView<'a, R> = MaterializedView<
    Option<OrderViewState>,
    OrderEvent,
    R,
    View<'a, Option<OrderViewState>, OrderEvent>,
    ErrorMessage,
>;

/// Convenient OrderSagaManager type alias - Saga pattern
pub type OrderSagaManager<'a, P> = SagaManager<
    OrderCommand,
    RestaurantEvent,
    P,
    Saga<'a, RestaurantEvent, OrderCommand>,
    ErrorMessage,
>;
