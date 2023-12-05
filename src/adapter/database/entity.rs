use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

/// DB ENTITY: Events are the core of the system. They are the source of truth for the system.
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct EventEntity {
    pub decider: String,
    pub decider_id: String,
    pub event: String,
    pub data: Value,
    pub event_id: Uuid,
    pub command_id: Option<Uuid>,
    pub previous_id: Option<Uuid>,
    pub r#final: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub offset: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewEventEntity {
    pub decider: String,
    pub decider_id: String,
    pub event: String,
    pub data: Value,
    pub event_id: Uuid,
    pub command_id: Option<Uuid>,
    pub previous_id: Option<Uuid>,
    pub r#final: bool,
}

/// DB ENTITY: Registered deciders and the respectful events that these deciders can publish (decider can publish and/or source its own state from these event types only)
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct DeciderEventEntity {
    pub decider: String,
    pub event: String,
}

/// DB ENTITY: Registered views that can subscribe to event streams.
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct ViewEntity {
    pub view: String,
    pub pooling_delay: i64,
    pub start_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DB ENTITY: Locks are used to prevent concurrent processing of the same decider events ont query side
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct LockEntity {
    pub view: String,
    pub decider_id: String,
    pub offset: i64,
    pub last_offset: i64,
    pub locked_until: DateTime<Utc>,
    pub offset_final: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DB ENTITY: Order view state
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct OrderEntity {
    pub id: String,
    pub data: Value,
}

/// DB ENTITY: Restaurant view state
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct RestaurantEntity {
    pub id: String,
    pub data: Value,
}
