use fmodel_rust::decider::Decider;

use crate::domain::api::{
    OrderCommand, OrderCreated, OrderEvent, OrderId, OrderLineItem, OrderNotCreated,
    OrderNotPrepared, OrderPrepared, OrderStatus, Reason, RestaurantId,
};

/// The state of the Order is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
}

/// A convenient type alias for the Order decider
pub type OrderDecider<'a> = Decider<'a, OrderCommand, Option<Order>, OrderEvent>;

/// Decider is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer.
pub fn order_decider<'a>() -> OrderDecider<'a> {
    Decider {
        // Decide new events based on the current state and the command
        // Exhaustive pattern matching on the command
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(command) => {
                if state.is_some() {
                    Ok(vec![OrderEvent::NotCreated(OrderNotCreated {
                        identifier: command.identifier.to_owned(),
                        restaurant_identifier: command.restaurant_identifier.to_owned(),
                        line_items: command.line_items.to_owned(),
                        reason: Reason("Order already exists".to_string()),
                    })])
                } else {
                    Ok(vec![OrderEvent::Created(OrderCreated {
                        identifier: command.identifier.to_owned(),
                        restaurant_identifier: command.restaurant_identifier.to_owned(),
                        status: OrderStatus::Created,
                        line_items: command.line_items.to_owned(),
                    })])
                }
            }
            OrderCommand::MarkAsPrepared(command) => {
                if state
                    .clone()
                    .is_some_and(|s| OrderStatus::Created == s.status)
                {
                    Ok(vec![OrderEvent::Prepared(OrderPrepared {
                        identifier: command.identifier.to_owned(),
                        status: OrderStatus::Prepared,
                    })])
                } else {
                    Ok(vec![OrderEvent::NotPrepared(OrderNotPrepared {
                        identifier: command.identifier.to_owned(),
                        reason: Reason("Order in the wrong status previously".to_string()),
                    })])
                }
            }
        }),
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            OrderEvent::Created(event) => Some(Order {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: event.restaurant_identifier.to_owned(),
                status: event.status.to_owned(),
                line_items: event.line_items.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Order, for example.
            OrderEvent::NotCreated(..) => state.clone(),
            OrderEvent::Prepared(event) => state.clone().map(|s| Order {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: s.restaurant_identifier,
                status: event.status.to_owned(),
                line_items: s.line_items,
            }),
            // On error event we choose NOT TO change the state of the Order, for example.
            OrderEvent::NotPrepared(..) => state.clone(),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}

#[cfg(test)]
/// Tests for the Order decider
mod order_decider_tests {
    use fmodel_rust::specification::DeciderTestSpecification;
    use uuid::Uuid;

    use crate::domain::api::{
        CreateOrder, MarkOrderAsPrepared, MenuItemId, MenuItemName, OrderCommand, OrderCreated,
        OrderEvent, OrderId, OrderLineItem, OrderLineItemId, OrderLineItemQuantity, OrderPrepared,
        OrderStatus, RestaurantId,
    };
    use crate::domain::order_decider::{order_decider, Order};

    #[test]
    fn create_order_test() {
        // The data
        let identifier = OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708207").unwrap());
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let order_line_item_id =
            OrderLineItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708209").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let line_items = vec![OrderLineItem {
            id: order_line_item_id,
            name: MenuItemName("Item 1".to_string()),
            quantity: OrderLineItemQuantity(1),
            menu_item_id,
        }];

        let create_order_command: OrderCommand = OrderCommand::Create(CreateOrder {
            identifier: identifier.clone(),
            restaurant_identifier: restaurant_identifier.clone(),
            line_items: line_items.clone(),
        });

        // ### EventSourced flavour ### - Test the decider: given EVENTS, when COMMAND, then NEW EVENTS
        DeciderTestSpecification::default()
            .for_decider(self::order_decider()) // Set the decider
            .given(vec![]) // no existing events
            .when(create_order_command.clone()) // Create an Order
            .then(vec![OrderEvent::Created(OrderCreated {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Created,
                line_items: line_items.clone(),
            })]);

        // ### StateStored flavour ### - Test the decider: given STATE, when COMMAND, then NEW STATE
        DeciderTestSpecification::default()
            .for_decider(self::order_decider()) // Set the decider
            .given_state(None) // no existing state
            .when(create_order_command.clone()) // Create an Order
            .then_state(Some(Order {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Created,
                line_items: line_items.clone(),
            }));
    }

    #[test]
    fn prepare_order_test() {
        // The data
        let identifier = OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708207").unwrap());
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let order_line_item_id =
            OrderLineItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708209").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let line_items = vec![OrderLineItem {
            id: order_line_item_id,
            name: MenuItemName("Item 1".to_string()),
            quantity: OrderLineItemQuantity(1),
            menu_item_id,
        }];

        let mark_order_as_prepared: OrderCommand =
            OrderCommand::MarkAsPrepared(MarkOrderAsPrepared {
                identifier: identifier.clone(),
            });

        // ### EventSourced flavour ### - Test the decider: given EVENTS, when COMMAND, then NEW EVENTS
        DeciderTestSpecification::default()
            .for_decider(self::order_decider()) // Set the decider
            .given(vec![OrderEvent::Created(OrderCreated {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Created,
                line_items: line_items.clone(),
            })]) // no existing events
            .when(mark_order_as_prepared.clone()) // Create an Order
            .then(vec![OrderEvent::Prepared(OrderPrepared {
                identifier: identifier.clone(),
                status: OrderStatus::Prepared,
            })]);

        // ### StateStored flavour ### - Test the decider: given STATE, when COMMAND, then NEW STATE
        DeciderTestSpecification::default()
            .for_decider(self::order_decider()) // Set the decider
            .given_state(Some(Some(Order {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Created,
                line_items: line_items.clone(),
            }))) // no existing state
            .when(mark_order_as_prepared.clone()) // Create an Order
            .then_state(Some(Order {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Prepared,
                line_items: line_items.clone(),
            }));
    }
}
