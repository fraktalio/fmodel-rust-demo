use crate::adapter::database::error::ErrorMessage;
use crate::adapter::repository::order_event_repository::OrderEventRepository;
use crate::application::api::OrderAggregate;
use crate::domain::api::OrderCommand;
use async_trait::async_trait;
use fmodel_rust::saga_manager::ActionPublisher;
use std::sync::Arc;

/// Order action publisher - used by the Saga Manager to publish actions/commands
pub struct OrderActionPublisher<'a> {
    pub order_aggregate: Arc<OrderAggregate<'a, OrderEventRepository>>,
}

/// Fmodel action publisher implementation fot the OrderActionPublisher
#[async_trait]
impl ActionPublisher<OrderCommand, ErrorMessage> for OrderActionPublisher<'_> {
    async fn publish(&self, commands: &[OrderCommand]) -> Result<Vec<OrderCommand>, ErrorMessage> {
        for command in commands {
            self.order_aggregate.handle(command).await?;
        }
        Ok(commands.to_vec())
    }
}
