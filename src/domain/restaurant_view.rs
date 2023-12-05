use fmodel_rust::view::View;
use serde::{Deserialize, Serialize};

use crate::domain::api::{RestaurantEvent, RestaurantId, RestaurantMenu, RestaurantName};

/// The state of the Restaurant View is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RestaurantViewState {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
}

/// A convenient type alias for the Restaurant view
type RestaurantView<'a> = View<'a, Option<RestaurantViewState>, RestaurantEvent>;

/// View represents the event handling algorithm. It belongs to the Domain layer.
pub fn restaurant_view<'a>() -> RestaurantView<'a> {
    View {
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            RestaurantEvent::Created(event) => Some(RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: event.name.to_owned(),
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the RestaurantView, for example.
            RestaurantEvent::NotCreated(..) => state.clone(),

            RestaurantEvent::MenuChanged(event) => state.clone().map(|s| RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the RestaurantView, for example.
            RestaurantEvent::MenuNotChanged(..) => state.clone(),

            RestaurantEvent::OrderPlaced(event) => state.clone().map(|s| RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: s.menu,
            }),
            // On error event we choose NOT TO change the state of the RestaurantView, for example.
            RestaurantEvent::OrderNotPlaced(..) => state.clone(),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}

#[cfg(test)]
/// Tests for the Restaurant view
mod restaurant_view_tests {
    use fmodel_rust::view::ViewStateComputation;
    use uuid::Uuid;

    use crate::domain::api::{
        MenuId, MenuItem, MenuItemId, MenuItemName, Money, RestaurantCreated, RestaurantEvent,
        RestaurantId, RestaurantMenu, RestaurantMenuChanged, RestaurantMenuCuisine, RestaurantName,
    };
    use crate::domain::restaurant_view::{restaurant_view, RestaurantView, RestaurantViewState};

    #[test]
    fn test() {
        // The Restaurant view
        let view: RestaurantView = restaurant_view();
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
        let restaurant_created: RestaurantEvent = RestaurantEvent::Created(RestaurantCreated {
            identifier: restaurant_identifier.clone(),
            name: RestaurantName("Restaurant 1".to_string()),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });
        let new_state = view.compute_new_state(None, &[&restaurant_created]);
        assert_eq!(
            new_state,
            Some(RestaurantViewState {
                identifier: restaurant_identifier.clone(),
                name: RestaurantName("Restaurant 1".to_string()),
                menu: RestaurantMenu {
                    menu_id: menu_id.clone(),
                    items: menu_items.clone(),
                    cuisine: RestaurantMenuCuisine::Vietnamese,
                },
            })
        );

        let menu_changed: RestaurantEvent = RestaurantEvent::MenuChanged(RestaurantMenuChanged {
            identifier: restaurant_identifier.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Japanese,
            },
        });
        let old_state = Some(RestaurantViewState {
            identifier: restaurant_identifier.clone(),
            name: RestaurantName("Restaurant 1".to_string()),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });
        let new_state = view.compute_new_state(Some(old_state), &[&menu_changed]);
        assert_eq!(
            new_state,
            Some(RestaurantViewState {
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
