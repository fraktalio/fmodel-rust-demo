use fmodel_rust::view::View;
use serde::{Deserialize, Serialize};

use crate::domain::api::{OrderEvent, OrderId, OrderLineItem, OrderStatus, RestaurantId};

/// The state of the Order is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrderViewState {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
}

/// A convenient type alias for the Order view
type OrderView<'a> = View<'a, Option<OrderViewState>, OrderEvent>;

/// View represents the event handling algorithm. It belongs to the Domain layer.
pub fn order_view<'a>() -> OrderView<'a> {
    View {
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            OrderEvent::Created(event) => Some(OrderViewState {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: event.restaurant_identifier.to_owned(),
                status: event.status.to_owned(),
                line_items: event.line_items.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Order, for example.
            OrderEvent::NotCreated(..) => state.clone(),

            OrderEvent::Prepared(event) => state.clone().map(|s| OrderViewState {
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
/// Tests for the Order view
mod order_view_tests {
    use fmodel_rust::view::ViewStateComputation;
    use uuid::Uuid;

    use crate::domain::api::{
        MenuItemId, MenuItemName, OrderCreated, OrderEvent, OrderId, OrderLineItem,
        OrderLineItemId, OrderLineItemQuantity, OrderPrepared, OrderStatus, RestaurantId,
    };
    use crate::domain::order_view::{order_view, OrderView, OrderViewState};

    #[test]
    fn test() {
        // The Order view
        let view: OrderView = order_view();
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

        let order_created: OrderEvent = OrderEvent::Created(OrderCreated {
            identifier: identifier.clone(),
            restaurant_identifier: restaurant_identifier.clone(),
            status: OrderStatus::Created,
            line_items: line_items.clone(),
        });

        let new_state = view.compute_new_state(None, &[&order_created]);
        assert_eq!(
            new_state,
            Some(OrderViewState {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Created,
                line_items: line_items.clone(),
            })
        );

        let order_prepared: OrderEvent = OrderEvent::Prepared(OrderPrepared {
            identifier: identifier.clone(),
            status: OrderStatus::Prepared,
        });
        let old_state = Some(OrderViewState {
            identifier: identifier.clone(),
            restaurant_identifier: restaurant_identifier.clone(),
            status: OrderStatus::Created,
            line_items: line_items.clone(),
        });
        let new_state = view.compute_new_state(Some(old_state), &[&order_prepared]);
        assert_eq!(
            new_state,
            Some(OrderViewState {
                identifier: identifier.clone(),
                restaurant_identifier: restaurant_identifier.clone(),
                status: OrderStatus::Prepared,
                line_items: line_items.clone(),
            })
        );
    }
}
