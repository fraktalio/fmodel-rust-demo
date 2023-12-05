use fmodel_rust::saga::Saga;

use crate::domain::api::{CreateOrder, OrderCommand, RestaurantEvent};

/// A convenient type alias for the Order saga
type OrderSaga<'a> = Saga<'a, RestaurantEvent, OrderCommand>;

/// The Order saga - represents the central point of control deciding what to execute next.
/// It is a function that takes an event and returns a list of commands.
pub fn order_saga<'a>() -> OrderSaga<'a> {
    Saga {
        react: Box::new(|event| match event {
            RestaurantEvent::OrderPlaced(event) => {
                vec![OrderCommand::Create(CreateOrder {
                    identifier: event.order_identifier.to_owned(),
                    restaurant_identifier: event.identifier.to_owned(),
                    line_items: event.line_items.to_owned(),
                })]
            }
            RestaurantEvent::OrderNotPlaced(..) => {
                vec![]
            }
            RestaurantEvent::NotCreated(..) => {
                vec![]
            }
            RestaurantEvent::MenuNotChanged(..) => {
                vec![]
            }
            RestaurantEvent::Created(..) => {
                vec![]
            }
            RestaurantEvent::MenuChanged(..) => {
                vec![]
            }
        }),
    }
}

#[cfg(test)]
/// Tests for the Order saga
mod order_saga_tests {
    use uuid::Uuid;

    use crate::domain::api::{
        CreateOrder, MenuItemId, MenuItemName, OrderCommand, OrderId, OrderLineItem,
        OrderLineItemId, OrderLineItemQuantity, OrderPlaced, RestaurantEvent, RestaurantId,
    };
    use crate::domain::order_saga::{order_saga, OrderSaga};

    #[test]
    fn test() {
        // The Order saga
        let saga: OrderSaga = order_saga();
        // The data
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());

        let order_placed_event = RestaurantEvent::OrderPlaced(OrderPlaced {
            identifier: restaurant_identifier.clone(),
            order_identifier: OrderId(
                Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708207").unwrap(),
            ),
            line_items: vec![OrderLineItem {
                id: OrderLineItemId(
                    Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708209").unwrap(),
                ),
                name: MenuItemName("Item 1".to_string()),
                quantity: OrderLineItemQuantity(1),
                menu_item_id: menu_item_id.clone(),
            }],
        });

        let create_order_command = OrderCommand::Create(CreateOrder {
            identifier: OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708207").unwrap()),
            restaurant_identifier: restaurant_identifier.clone(),
            line_items: vec![OrderLineItem {
                id: OrderLineItemId(
                    Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708209").unwrap(),
                ),
                name: MenuItemName("Item 1".to_string()),
                quantity: OrderLineItemQuantity(1),
                menu_item_id: menu_item_id.clone(),
            }],
        });

        let commands = (saga.react)(&order_placed_event);
        assert_eq!(commands, vec![create_order_command]);
    }
}
