use fmodel_rust::Sum;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ########################################################
// #################### Value Objects #####################
// ########################################################

// The 'newtype' pattern is typical in functional programming. In Haskell, this pattern is supported via the 'newtype' declaration, which allows the programmer to define a new type identical to an existing one except for its name. This is useful for creating type-safe abstractions, enabling the programmer to enforce stronger type constraints on using specific values.
// Similarly, in Rust, the 'newtype' idiom brings compile-time guarantees that the correct value type is supplied. The 'newtype' is a struct that wraps a single value and provides a new type for that value. A 'newtype' is the same as the underlying type at runtime, so it will not introduce any performance overhead.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RestaurantId(pub Uuid);
impl fmt::Display for RestaurantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RestaurantName(pub String);
impl fmt::Display for RestaurantName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OrderId(pub Uuid);
impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Reason(pub String);
impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Money(pub f64);
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MenuId(pub Uuid);
impl fmt::Display for MenuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MenuItemId(pub Uuid);
impl fmt::Display for MenuItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MenuItemName(pub String);
impl fmt::Display for MenuItemName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OrderLineItemId(pub Uuid);
impl fmt::Display for OrderLineItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OrderLineItemQuantity(pub u32);
impl fmt::Display for OrderLineItemQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MenuItem {
    pub id: MenuItemId,
    pub name: MenuItemName,
    pub price: Money,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RestaurantMenuCuisine {
    Italian,
    Indian,
    Chinese,
    Japanese,
    American,
    Mexican,
    French,
    Thai,
    Vietnamese,
    Greek,
    Korean,
    Spanish,
    Lebanese,
    Turkish,
    Ethiopian,
    Moroccan,
    Egyptian,
    Brazilian,
    Polish,
    German,
    British,
    Irish,
    Other,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RestaurantMenu {
    pub menu_id: MenuId,
    pub items: Vec<MenuItem>,
    pub cuisine: RestaurantMenuCuisine,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OrderLineItem {
    pub id: OrderLineItemId,
    pub quantity: OrderLineItemQuantity,
    pub menu_item_id: MenuItemId,
    pub name: MenuItemName,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum OrderStatus {
    Created,
    Prepared,
    Cancelled,
    Rejected,
}

// ########################################################
// ####################### COMMANDS #######################
// ########################################################
/// Intent/Command to create a new restaurant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateRestaurant {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
}

/// Intent/Command to change the menu of a restaurant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ChangeRestaurantMenu {
    pub identifier: RestaurantId,
    pub menu: RestaurantMenu,
}

/// Intent/Command to place an order at a restaurant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PlaceOrder {
    pub identifier: RestaurantId,
    pub order_identifier: OrderId,
    pub line_items: Vec<OrderLineItem>,
}

/// All possible command variants that could be sent to a restaurant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum RestaurantCommand {
    CreateRestaurant(CreateRestaurant),
    ChangeMenu(ChangeRestaurantMenu),
    PlaceOrder(PlaceOrder),
}

/// Intent/Command to create a new order
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CreateOrder {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub line_items: Vec<OrderLineItem>,
}

/// Intent/Command to mark an order as prepared
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MarkOrderAsPrepared {
    pub identifier: OrderId,
}

/// All possible command variants that could be sent to an order
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum OrderCommand {
    Create(CreateOrder),
    MarkAsPrepared(MarkOrderAsPrepared),
}

// ########################################################
// ######################## EVENTS ########################
// ########################################################

/// Fact/Event that a restaurant was created
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RestaurantCreated {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
}

/// Fact/Event that a restaurant was not created (with reason)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RestaurantNotCreated {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
    pub reason: Reason,
}

/// Fact/Event that a restaurant's menu was changed
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RestaurantMenuChanged {
    pub identifier: RestaurantId,
    pub menu: RestaurantMenu,
}

/// Fact/Event that a restaurant's menu was not changed (with reason)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RestaurantMenuNotChanged {
    pub identifier: RestaurantId,
    pub menu: RestaurantMenu,
    pub reason: Reason,
}

/// Fact/Event that an order was placed
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderPlaced {
    pub identifier: RestaurantId,
    pub order_identifier: OrderId,
    pub line_items: Vec<OrderLineItem>,
}

/// Fact/Event that an order was not placed (with reason)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderNotPlaced {
    pub identifier: RestaurantId,
    pub order_identifier: OrderId,
    pub line_items: Vec<OrderLineItem>,
    pub reason: Reason,
}

/// All possible event variants that could be used to update a restaurant
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum RestaurantEvent {
    Created(RestaurantCreated),
    NotCreated(RestaurantNotCreated),
    MenuChanged(RestaurantMenuChanged),
    MenuNotChanged(RestaurantMenuNotChanged),
    OrderPlaced(OrderPlaced),
    OrderNotPlaced(OrderNotPlaced),
}

/// Fact/Event that an order was created
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderCreated {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
}

/// Fact/Event that an order was not created (with reason)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderNotCreated {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub line_items: Vec<OrderLineItem>,
    pub reason: Reason,
}

/// Fact/Event that an order was prepared
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderPrepared {
    pub identifier: OrderId,
    pub status: OrderStatus,
}

/// Fact/Event that an order was not prepared (with reason)
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OrderNotPrepared {
    pub identifier: OrderId,
    pub reason: Reason,
}

/// All possible event variants that could be used to update an order
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum OrderEvent {
    Created(OrderCreated),
    NotCreated(OrderNotCreated),
    Prepared(OrderPrepared),
    NotPrepared(OrderNotPrepared),
}

/// All possible command variants that could be sent
pub type Command = Sum<RestaurantCommand, OrderCommand>;

/// All possible event variants that could be used
pub type Event = Sum<RestaurantEvent, OrderEvent>;

// ########################################################
// ####################     TRAITS      ###################
// ########################################################

/// ###### Trait to get the identifier of a message #######
pub trait Identifier {
    fn identifier(&self) -> String;
}

impl Identifier for RestaurantCommand {
    fn identifier(&self) -> String {
        match self {
            RestaurantCommand::CreateRestaurant(command) => command.identifier.to_string(),
            RestaurantCommand::ChangeMenu(command) => command.identifier.to_string(),
            RestaurantCommand::PlaceOrder(command) => command.identifier.to_string(),
        }
    }
}

impl Identifier for OrderCommand {
    fn identifier(&self) -> String {
        match self {
            OrderCommand::Create(command) => command.identifier.to_string(),
            OrderCommand::MarkAsPrepared(command) => command.identifier.to_string(),
        }
    }
}

impl Identifier for Command {
    fn identifier(&self) -> String {
        match self {
            Command::First(command) => command.identifier(),
            Command::Second(command) => command.identifier(),
        }
    }
}

impl Identifier for RestaurantEvent {
    fn identifier(&self) -> String {
        match self {
            RestaurantEvent::Created(event) => event.identifier.to_string(),
            RestaurantEvent::NotCreated(event) => event.identifier.to_string(),
            RestaurantEvent::MenuChanged(event) => event.identifier.to_string(),
            RestaurantEvent::MenuNotChanged(event) => event.identifier.to_string(),
            RestaurantEvent::OrderPlaced(event) => event.identifier.to_string(),
            RestaurantEvent::OrderNotPlaced(event) => event.identifier.to_string(),
        }
    }
}

impl Identifier for OrderEvent {
    fn identifier(&self) -> String {
        match self {
            OrderEvent::Created(event) => event.identifier.to_string(),
            OrderEvent::NotCreated(event) => event.identifier.to_string(),
            OrderEvent::Prepared(event) => event.identifier.to_string(),
            OrderEvent::NotPrepared(event) => event.identifier.to_string(),
        }
    }
}

impl Identifier for Event {
    fn identifier(&self) -> String {
        match self {
            Event::First(event) => event.identifier(),
            Event::Second(event) => event.identifier(),
        }
    }
}

/// ###### Trait to get the decider name/type of a message #######
pub trait DeciderName {
    fn decider_name(&self) -> String;
}

impl DeciderName for RestaurantEvent {
    fn decider_name(&self) -> String {
        match self {
            RestaurantEvent::Created(_) => "Restaurant".to_string(),
            RestaurantEvent::NotCreated(_) => "Restaurant".to_string(),
            RestaurantEvent::MenuChanged(_) => "Restaurant".to_string(),
            RestaurantEvent::MenuNotChanged(_) => "Restaurant".to_string(),
            RestaurantEvent::OrderPlaced(_) => "Restaurant".to_string(),
            RestaurantEvent::OrderNotPlaced(_) => "Restaurant".to_string(),
        }
    }
}

impl DeciderName for OrderEvent {
    fn decider_name(&self) -> String {
        match self {
            OrderEvent::Created(_) => "Order".to_string(),
            OrderEvent::NotCreated(_) => "Order".to_string(),
            OrderEvent::Prepared(_) => "Order".to_string(),
            OrderEvent::NotPrepared(_) => "Order".to_string(),
        }
    }
}

impl DeciderName for Event {
    fn decider_name(&self) -> String {
        match self {
            Event::First(event) => event.decider_name(),
            Event::Second(event) => event.decider_name(),
        }
    }
}

/// ###### Trait to get the decider name/type of an event #######
pub trait EventName {
    fn event_name(&self) -> String;
}

impl EventName for RestaurantEvent {
    fn event_name(&self) -> String {
        match self {
            RestaurantEvent::Created(_) => "RestaurantCreated".to_string(),
            RestaurantEvent::NotCreated(_) => "RestaurantNotCreated".to_string(),
            RestaurantEvent::MenuChanged(_) => "RestaurantMenuChanged".to_string(),
            RestaurantEvent::MenuNotChanged(_) => "RestaurantMenuNotChanged".to_string(),
            RestaurantEvent::OrderPlaced(_) => "OrderPlaced".to_string(),
            RestaurantEvent::OrderNotPlaced(_) => "OrderNotPlaced".to_string(),
        }
    }
}

impl EventName for OrderEvent {
    fn event_name(&self) -> String {
        match self {
            OrderEvent::Created(_) => "OrderCreated".to_string(),
            OrderEvent::NotCreated(_) => "OrderNotCreated".to_string(),
            OrderEvent::Prepared(_) => "OrderPrepared".to_string(),
            OrderEvent::NotPrepared(_) => "OrderNotPrepared".to_string(),
        }
    }
}

impl EventName for Event {
    fn event_name(&self) -> String {
        match self {
            Event::First(event) => event.event_name(),
            Event::Second(event) => event.event_name(),
        }
    }
}
