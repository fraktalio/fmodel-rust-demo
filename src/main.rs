#![deny(missing_docs)]

//! This is an entry point for the application.

use std::env::var;
use std::process::exit;
use std::sync::{Arc, Once};
use std::time::Duration;

use crate::adapter::event_stream::saga_stream::stream_events_to_saga;
use crate::adapter::event_stream::view_stream::stream_events_to_view;
use crate::adapter::publisher::order_action_publisher::OrderActionPublisher;
use crate::adapter::repository::event_repository::AggregateEventRepository;
use crate::adapter::repository::order_view_state_repository::OrderViewStateRepository;
use crate::adapter::repository::restaurant_view_state_repository::RestaurantViewStateRepository;
use crate::adapter::web::handler;
use crate::application::api::Application;
use crate::domain::order_decider::order_decider;
use crate::domain::order_saga::order_saga;
use crate::domain::order_view::order_view;
use crate::domain::restaurant_decider::restaurant_decider;
use crate::domain::restaurant_view::restaurant_view;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use adapter::database::error::ErrorMessage;

use fmodel_rust::aggregate::EventSourcedAggregate;
use fmodel_rust::materialized_view::MaterializedView;
use fmodel_rust::saga_manager::SagaManager;
use sqlx::{migrate, postgres::PgPoolOptions, Pool, Postgres};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::layer::SubscriberExt;

mod adapter;
mod application;
mod domain;

static INIT: Once = Once::new();

/// Application state - database connection pool
pub struct Database {
    db: Pool<Postgres>,
}

/// Database URL environment variable
pub const DATABASE_URL: &str = "DATABASE_URL";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    init_logger();
    // Initialize the database
    let database_url = var(DATABASE_URL).expect("DATABASE_URL must be set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            tracing::error!("🔥 Failed to connect to the database: {err:?}");
            exit(1);
        }
    };

    // Run the database migrations
    match migrate!().run(&pool).await {
        Ok(_) => {
            tracing::info!("✅ Migration is successful!");
        }
        Err(err) => {
            tracing::error!("🔥 Migration failed: {err:?}");
            exit(1);
        }
    }

    // ##### COMMAND SIDE - create an aggregate per decider - distributed scenario #####
    // Create the order repository - command side
    let order_event_repository = AggregateEventRepository::new(Database { db: pool.clone() });
    // Create the restaurant repository - command side
    let restaurant_event_repository = AggregateEventRepository::new(Database { db: pool.clone() });
    // Create the restaurant aggregate - command side
    let restaurant_aggregate = Arc::new(EventSourcedAggregate::new(
        restaurant_event_repository,
        // Decider
        // Error type needs to match the error type of the aggregate
        restaurant_decider().map_error(|_| ErrorMessage {
            message: "Restaurant decider error".to_string(),
        }),
    ));
    // Create the order aggregate - command side
    let order_aggregate = Arc::new(EventSourcedAggregate::new(
        order_event_repository,
        // Decider
        // Error type needs to match the error type of the aggregate
        order_decider().map_error(|_| ErrorMessage {
            message: "Order decider error".to_string(),
        }),
    ));

    // ###### QUERY SIDE ######
    // Create the restaurant query handler -
    let restaurant_query_handler =
        RestaurantViewStateRepository::new(Database { db: pool.clone() });
    // Create the restaurant view state repository - query side
    let restaurant_view_state_repository =
        RestaurantViewStateRepository::new(Database { db: pool.clone() });
    // Create the restaurant materialized view - query side - handles the events from the event store and projects them into the denormalized state
    let restaurant_materialized_view = Arc::new(MaterializedView::new(
        restaurant_view_state_repository,
        restaurant_view(),
    ));
    // Create the order query handler - query side
    let order_query_handler = OrderViewStateRepository::new(Database { db: pool.clone() });
    // Create the order view state repository - query side
    let order_view_state_repository = OrderViewStateRepository::new(Database { db: pool.clone() });
    // Create the order materialized view - query side - handles the events from the event store and projects them into the denormalized state
    let order_materialized_view = Arc::new(MaterializedView::new(
        order_view_state_repository,
        order_view(),
    ));

    // Action Publisher for the Saga manager
    let order_action_publisher = OrderActionPublisher {
        order_aggregate: order_aggregate.clone(),
    };
    // Saga manager
    let order_saga_manager = Arc::new(SagaManager::new(order_action_publisher, order_saga()));

    // Start a background task for all the event handling and processing
    // 1. stop signal for canceling background task
    let background_task_cancellation = CancellationToken::new();
    let background_task_cancellation_clone = background_task_cancellation.clone();
    // 2. Spawn the background task
    let background_task = actix_web::rt::spawn(async move {
        let db = Database { db: pool.clone() };
        loop {
            match stream_events_to_view(
                restaurant_materialized_view.clone(),
                order_materialized_view.clone(),
                &db,
            )
            .await
            {
                Ok(_) => {}
                Err(error) => {
                    tracing::error!("###  View Stream closed with error: {} ###", error.message);
                    break;
                }
            }

            match stream_events_to_saga(order_saga_manager.clone(), &db).await {
                Ok(_) => {}
                Err(error) => {
                    tracing::error!("###  Saga Stream closed with error: {} ###", error.message);
                    break;
                }
            }

            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {
                    tracing::debug!("### Waiting for 1 second ###");
                    continue;
                }

                _ = background_task_cancellation_clone.cancelled() => {
                    tracing::info!("### Gracefully shutting event handler ###");
                    break;
                }
            };
        }
    });

    // Note: web::Data created _outside_ HttpServer::new closure
    // We will write an application with mutable, shared state. First, we define our state and create our handler function
    let application = web::Data::new(Application {
        restaurant_aggregate: restaurant_aggregate.clone(),
        order_aggregate: order_aggregate.clone(),
        restaurant_query_handler,
        order_query_handler,
    });
    // Start the HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .app_data(application.clone())
            .configure(handler::config)
            .wrap(Logger::default())
            .wrap(cors)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;

    background_task_cancellation.cancel();

    background_task.await?;
    tracing::info!("### Application gracefully shut down ###");

    Ok(())
}

fn init_logger() {
    INIT.call_once(|| {
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()), // 👈 Set default to `info`
            )
            .with(tracing_subscriber::fmt::layer());

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}
