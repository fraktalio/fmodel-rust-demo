use fmodel_rust::decider::Decider;

use crate::domain::api::{
    OrderNotPlaced, OrderPlaced, Reason, RestaurantCommand, RestaurantCreated, RestaurantEvent,
    RestaurantId, RestaurantMenu, RestaurantMenuChanged, RestaurantMenuNotChanged, RestaurantName,
    RestaurantNotCreated,
};

/// The state of the Restaurant is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug)]
pub struct Restaurant {
    identifier: RestaurantId,
    name: RestaurantName,
    menu: RestaurantMenu,
}

/// A convenient type alias for the Restaurant decider
pub type RestaurantDecider<'a> =
    Decider<'a, RestaurantCommand, Option<Restaurant>, RestaurantEvent>;

/// Decider is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer.
pub fn restaurant_decider<'a>() -> RestaurantDecider<'a> {
    Decider {
        // Decide new events based on the current state and the command
        // Exhaustive pattern matching on the command
        decide: Box::new(|command, state| match command {
            RestaurantCommand::CreateRestaurant(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::NotCreated(RestaurantNotCreated {
                        identifier: command.identifier.to_owned(),
                        name: command.name.to_owned(),
                        menu: command.menu.to_owned(),
                        reason: Reason("Restaurant already exists".to_string()),
                    })]
                } else {
                    vec![RestaurantEvent::Created(RestaurantCreated {
                        identifier: command.identifier.to_owned(),
                        name: command.name.to_owned(),
                        menu: command.menu.to_owned(),
                    })]
                }
            }
            RestaurantCommand::ChangeMenu(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::MenuChanged(RestaurantMenuChanged {
                        identifier: command.identifier.to_owned(),
                        menu: command.menu.to_owned(),
                    })]
                } else {
                    vec![RestaurantEvent::MenuNotChanged(RestaurantMenuNotChanged {
                        identifier: command.identifier.to_owned(),
                        menu: command.menu.to_owned(),
                        reason: Reason("Restaurant does not exist".to_string()),
                    })]
                }
            }
            RestaurantCommand::PlaceOrder(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::OrderPlaced(OrderPlaced {
                        identifier: command.identifier.to_owned(),
                        order_identifier: command.order_identifier.to_owned(),
                        line_items: command.line_items.to_owned(),
                    })]
                } else {
                    vec![RestaurantEvent::OrderNotPlaced(OrderNotPlaced {
                        identifier: command.identifier.to_owned(),
                        order_identifier: command.order_identifier.to_owned(),
                        line_items: command.line_items.to_owned(),
                        reason: Reason("Restaurant does not exist".to_string()),
                    })]
                }
            }
        }),
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            RestaurantEvent::Created(event) => Some(Restaurant {
                identifier: event.identifier.to_owned(),
                name: event.name.to_owned(),
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::NotCreated(..) => state.clone(),

            RestaurantEvent::MenuChanged(event) => state.clone().map(|s| Restaurant {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::MenuNotChanged(..) => state.clone(),

            RestaurantEvent::OrderPlaced(event) => state.clone().map(|s| Restaurant {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: s.menu,
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::OrderNotPlaced(..) => state.clone(),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}

#[cfg(test)]
/// Tests for the Restaurant decider
mod restaurant_decider_tests {
    use fmodel_rust::decider::{EventComputation, StateComputation};
    use uuid::Uuid;

    use crate::domain::api::{
        ChangeRestaurantMenu, CreateRestaurant, MenuId, MenuItem, MenuItemId, MenuItemName, Money,
        RestaurantCommand, RestaurantCreated, RestaurantEvent, RestaurantId, RestaurantMenu,
        RestaurantMenuChanged, RestaurantMenuCuisine, RestaurantName,
    };
    use crate::domain::restaurant_decider::{restaurant_decider, Restaurant, RestaurantDecider};

    #[test]
    fn test() {
        // The Restaurant decider
        let decider: RestaurantDecider = restaurant_decider();
        // The data
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id,
            name: MenuItemName("Item 1".to_string()),
            price: Money(100.1),
        }];

        // The command to create an order - CreateOrder
        let create_restaurant_command: RestaurantCommand =
            RestaurantCommand::CreateRestaurant(CreateRestaurant {
                identifier: restaurant_identifier.clone(),
                name: RestaurantName("Restaurant 1".to_string()),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Vietnamese,
                },
            });
        // ### EventSourced flavour ### - Test the decider: given EVENTS, when COMMAND, then NEW EVENTS
        let new_events = decider.compute_new_events(&[], &create_restaurant_command);
        assert_eq!(
            new_events,
            [RestaurantEvent::Created(RestaurantCreated {
                identifier: restaurant_identifier.clone(),
                name: RestaurantName("Restaurant 1".to_string()),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Vietnamese,
                },
            })]
        );
        // ### StateStored flavour ### - Test the decider: given STATE, when COMMAND, then NEW STATE
        let new_state = decider.compute_new_state(None, &create_restaurant_command);
        assert_eq!(
            new_state,
            Some(Restaurant {
                identifier: restaurant_identifier.clone(),
                name: RestaurantName("Restaurant 1".to_string()),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Vietnamese,
                },
            })
        );

        // The command to create an order - MarkOrderAsPrepared
        let change_restaurant_menu: RestaurantCommand =
            RestaurantCommand::ChangeMenu(ChangeRestaurantMenu {
                identifier: restaurant_identifier.clone(),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Japanese,
                },
            });
        // ### EventSourced flavour ### - Test the decider: given EVENTS, when COMMAND, then NEW EVENTS
        let old_events = vec![RestaurantEvent::Created(RestaurantCreated {
            identifier: restaurant_identifier.clone(),
            name: RestaurantName("Restaurant 1".to_string()),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        })];
        let new_events = decider.compute_new_events(&old_events, &change_restaurant_menu);
        assert_eq!(
            new_events,
            [RestaurantEvent::MenuChanged(RestaurantMenuChanged {
                identifier: restaurant_identifier.clone(),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Japanese,
                },
            })]
        );

        // ### StateStored flavour ### - Test the decider: given STATE, when COMMAND, then NEW STATE
        let old_state = Some(Restaurant {
            identifier: restaurant_identifier.clone(),
            name: RestaurantName("Restaurant 1".to_string()),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });
        let new_state = decider.compute_new_state(Some(old_state), &change_restaurant_menu);
        assert_eq!(
            new_state,
            Some(Restaurant {
                identifier: restaurant_identifier.clone(),
                name: RestaurantName("Restaurant 1".to_string()),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Japanese,
                },
            })
        );
    }
}
