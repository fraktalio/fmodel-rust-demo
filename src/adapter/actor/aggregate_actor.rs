use crate::adapter::database::error::ErrorMessage;
use crate::adapter::repository::order_event_repository::OrderEventRepository;
use crate::adapter::repository::restaurant_event_repository::RestaurantEventRepository;
use crate::application::api::{OrderAggregate, RestaurantAggregate};
use crate::domain::api::{OrderCommand, RestaurantCommand};
use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use chashmap::CHashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Summing all the variants of the RestaurantCommand and OrderCommand / Command enum
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// First variant
    First(RestaurantCommand),
    /// Second variant
    Second(OrderCommand),
}

impl Message for Command {
    type Result = Result<(), ErrorMessage>;
}

/// Id trait - A trait to get the id/identifier of the command
trait Id {
    fn id(&self) -> String;
}

impl Id for Command {
    /// Get the id/identifier of the command
    fn id(&self) -> String {
        match self {
            Command::First(restaurant_command) => match restaurant_command {
                RestaurantCommand::CreateRestaurant(restaurant) => {
                    restaurant.identifier.to_string()
                }
                RestaurantCommand::PlaceOrder(restaurant) => restaurant.identifier.to_string(),
                RestaurantCommand::ChangeMenu(restaurant) => restaurant.identifier.to_string(),
            },
            Command::Second(order_command) => match order_command {
                OrderCommand::Create(order) => order.identifier.to_string(),
                OrderCommand::MarkAsPrepared(order) => order.identifier.to_string(),
            },
        }
    }
}

/// AggregateActor struct - An actor that handles commands
struct AggregateActor {
    restaurant_aggregate: Arc<RestaurantAggregate<'static, RestaurantEventRepository>>,
    order_aggregate: Arc<OrderAggregate<'static, OrderEventRepository>>,
}

impl Actor for AggregateActor {
    type Context = Context<Self>;
}

impl Handler<Command> for AggregateActor {
    type Result = ResponseFuture<Result<(), ErrorMessage>>;

    /// Handle the command
    fn handle(&mut self, msg: Command, _: &mut Self::Context) -> Self::Result {
        let restaurant_aggregate = self.restaurant_aggregate.clone();
        let order_aggregate = self.order_aggregate.clone();

        Box::pin(async move {
            match msg {
                Command::First(restaurant_command) => restaurant_aggregate
                    .handle(&restaurant_command)
                    .await
                    .map(|_| ()),
                Command::Second(order_command) => {
                    order_aggregate.handle(&order_command).await.map(|_| ())
                }
            }
        })
    }
}

/// Actor that implements consistent hashing algorithm. It routes the commands to the target actor based on the hash of the command's key.
struct ConsistentHashingActor {
    actors: CHashMap<usize, Addr<AggregateActor>>,
    num_actors: usize,
}

impl ConsistentHashingActor {
    /// Calculate the target actor based on the hash of the command's key
    fn calculate_target_actor(&self, key: &str) -> usize {
        // Use simple hash and modulo to calculate the target actor
        key.chars().fold(0, |acc, c| acc + c as usize) % self.num_actors
    }
}

impl Actor for ConsistentHashingActor {
    type Context = Context<Self>;
}

impl Handler<Command> for ConsistentHashingActor {
    type Result = Result<(), ErrorMessage>;
    /// Handle the command
    fn handle(&mut self, msg: Command, _: &mut Self::Context) -> Result<(), ErrorMessage> {
        // Calculate the target actor based on the hash of the command's key
        let target_actor = self.calculate_target_actor(&msg.id());

        if let Some(actor) = self.actors.get(&target_actor) {
            actor.try_send(msg).map_err(|err| ErrorMessage {
                message: format!("Failed to send the message to the target actor: {:?}", err),
            })
        } else {
            Err(ErrorMessage {
                message: "Failed to send the message to the target actor".to_string(),
            })
        }
    }
}
