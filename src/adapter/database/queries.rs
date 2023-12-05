use actix_web::web;
use web::Data;

use crate::adapter::database::entity::{
    DeciderEventEntity, EventEntity, LockEntity, NewEventEntity, OrderEntity, RestaurantEntity,
    ViewEntity,
};
use crate::adapter::database::error::ErrorMessage;
use crate::Database;

// ############################### COMMAND SIDE ###############################

/// DB: Register the type of event(s) that this `decider` is able to publish/store
/// Event can not be inserted into `event` table without the matching event being registered previously. It is controlled by the 'Foreign Key' constraint on the `event` table
#[allow(dead_code)]
pub async fn register_decider(
    event: &String,
    decider: &String,
    app: &Data<Database>,
) -> Result<DeciderEventEntity, ErrorMessage> {
    sqlx::query_as!(
        DeciderEventEntity,
        "INSERT INTO deciders (decider, event) VALUES ($1, $2) RETURNING *;",
        event,
        decider
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Get events by `decider_id`
/// Used by the `Decider/Entity` to get list of events from where it can source its own state
#[allow(dead_code)]
pub async fn list_events(
    decider_id: &String,
    app: &Database,
) -> Result<Vec<EventEntity>, ErrorMessage> {
    sqlx::query_as!(
        EventEntity,
        "SELECT * FROM events WHERE decider_id = $1 ORDER BY events.offset",
        decider_id
    )
    .fetch_all(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Get the latest event by `decider_id`
/// Used by the `Decider/Entity` to get the latest event from where it can get the latest version of its own state / use it for optimistic locking
#[allow(dead_code)]
pub async fn get_latest_event(
    decider_id: &String,
    app: &Database,
) -> Result<EventEntity, ErrorMessage> {
    sqlx::query_as!(
        EventEntity,
        "SELECT * FROM events WHERE decider_id = $1 ORDER BY events.offset DESC LIMIT 1",
        decider_id
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Append/Insert new 'event'
#[allow(dead_code)]
pub async fn append_event(
    event: &NewEventEntity,
    app: &Database,
) -> Result<EventEntity, ErrorMessage> {
    sqlx::query_as!(
        EventEntity,
        "INSERT INTO events (event, event_id, decider, decider_id, data, command_id, previous_id, final)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *",
        event.event,
        event.event_id,
        event.decider,
        event.decider_id,
        event.data,
        event.command_id,
        event.previous_id,
        event.r#final
    )
        .fetch_one(&app.db)
        .await
        .map_err(|e| ErrorMessage {
            message: e.to_string(),
        })
}

// ############################### QUERY SIDE ###############################

/// DB: Register a new view
#[allow(dead_code)]
pub async fn register_view(
    view: &String,
    pooling_delay: &i64,
    app: &Data<Database>,
) -> Result<ViewEntity, ErrorMessage> {
    sqlx::query_as!(
        ViewEntity,
        "INSERT INTO views (view, pooling_delay)
                VALUES ($1, $2)
                RETURNING *;",
        view,
        pooling_delay
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Stream events from the `event` table to the materialized view
pub async fn stream_events(
    view: &String,
    app: &Database,
) -> Result<Option<EventEntity>, ErrorMessage> {
    sqlx::query_as::<_, EventEntity>("SELECT * FROM stream_events($1)")
        .bind(view)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| ErrorMessage {
            message: e.to_string(),
        })
}

/// DB: Ack that event was processed successfully
pub async fn ack_event(
    offset: &i64,
    view: &String,
    decider_id: &String,
    app: &Database,
) -> Result<LockEntity, ErrorMessage> {
    sqlx::query_as!(
        LockEntity,
        "UPDATE locks
            SET locked_until = NOW(), -- locked = false,
                last_offset = $1
            WHERE view = $2
            AND decider_id = $3
            RETURNING *;",
        offset,
        view,
        decider_id
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Nack that event was not processed successfully
pub async fn nack_event(
    view: &String,
    decider_id: &String,
    app: &Database,
) -> Result<LockEntity, ErrorMessage> {
    sqlx::query_as!(
        LockEntity,
        "UPDATE locks
            SET locked_until = NOW() -- locked = false
            WHERE view = $1
            AND decider_id = $2
            RETURNING *;",
        view,
        decider_id
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Get the Order view state by `id`
pub async fn get_order(id: &String, app: &Database) -> Result<Option<OrderEntity>, ErrorMessage> {
    sqlx::query_as!(OrderEntity, "SELECT * FROM orders WHERE id = $1", id)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| ErrorMessage {
            message: e.to_string(),
        })
}

/// DB: Get all the Order view states
pub async fn get_all_orders(app: &Database) -> Result<Vec<OrderEntity>, ErrorMessage> {
    sqlx::query_as!(OrderEntity, "SELECT * FROM orders",)
        .fetch_all(&app.db)
        .await
        .map_err(|e| ErrorMessage {
            message: e.to_string(),
        })
}

/// DB: Insert/Update the Order view state
pub async fn upsert_order(
    order: &OrderEntity,
    app: &Database,
) -> Result<OrderEntity, ErrorMessage> {
    sqlx::query_as!(
        OrderEntity,
        "INSERT INTO orders (id, data)
            VALUES ($1, $2)
         ON CONFLICT ON CONSTRAINT orders_pkey
         DO UPDATE SET data = EXCLUDED.data
            RETURNING *",
        order.id,
        order.data,
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Get the Restaurant view state by `id`
pub async fn get_restaurant(
    id: &String,
    app: &Database,
) -> Result<Option<RestaurantEntity>, ErrorMessage> {
    sqlx::query_as!(
        RestaurantEntity,
        "SELECT * FROM restaurants WHERE id = $1",
        id
    )
    .fetch_optional(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}

/// DB: Get all the Restaurant view states
pub async fn get_all_restaurants(app: &Database) -> Result<Vec<RestaurantEntity>, ErrorMessage> {
    sqlx::query_as!(RestaurantEntity, "SELECT * FROM restaurants",)
        .fetch_all(&app.db)
        .await
        .map_err(|e| ErrorMessage {
            message: e.to_string(),
        })
}

/// DB: Insert/Update the Restaurant view state
pub async fn upsert_restaurant(
    restaurant: &RestaurantEntity,
    app: &Database,
) -> Result<RestaurantEntity, ErrorMessage> {
    sqlx::query_as!(
        RestaurantEntity,
        "INSERT INTO restaurants (id, data)
            VALUES ($1, $2)
         ON CONFLICT ON CONSTRAINT restaurants_pkey
         DO UPDATE SET data = EXCLUDED.data
         RETURNING *",
        restaurant.id,
        restaurant.data,
    )
    .fetch_one(&app.db)
    .await
    .map_err(|e| ErrorMessage {
        message: e.to_string(),
    })
}
