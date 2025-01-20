use crate::adapter::repository::event_repository::AggregateEventRepository;
use crate::adapter::repository::order_view_state_repository::OrderViewStateRepository;
use crate::adapter::repository::restaurant_view_state_repository::RestaurantViewStateRepository;
use crate::application::api::{Application, OrderQueryHandler, RestaurantQueryHandler};
use crate::domain::api::{OrderCommand, RestaurantCommand};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

#[get("/healthchecker")]
async fn health_checker_handler(
    _application: web::Data<
        Application<
            '_,
            AggregateEventRepository,
            AggregateEventRepository,
            OrderViewStateRepository,
            RestaurantViewStateRepository,
        >,
    >,
) -> impl Responder {
    const MESSAGE: &str = "Fmodel demo is running!";

    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

#[post("/commands/order")]
async fn order_command_handler(
    command: web::Json<OrderCommand>,
    application: web::Data<
        Application<
            '_,
            AggregateEventRepository,
            AggregateEventRepository,
            OrderViewStateRepository,
            RestaurantViewStateRepository,
        >,
    >,
) -> impl Responder {
    let result = application
        .order_aggregate
        .handle(&command.into_inner())
        .await;

    match result {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(err) => HttpResponse::InternalServerError().json(json!(err)),
    }
}

#[get("/queries/order")]
async fn get_all_orders_handler(
    application: web::Data<
        Application<
            '_,
            AggregateEventRepository,
            AggregateEventRepository,
            OrderViewStateRepository,
            RestaurantViewStateRepository,
        >,
    >,
) -> impl Responder {
    let result = application.order_query_handler.get_all_orders().await;

    match result {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(err) => HttpResponse::InternalServerError().json(json!(err)),
    }
}

#[post("/commands/restaurant")]
async fn restaurant_command_handler(
    command: web::Json<RestaurantCommand>,
    application: web::Data<
        Application<
            '_,
            AggregateEventRepository,
            AggregateEventRepository,
            OrderViewStateRepository,
            RestaurantViewStateRepository,
        >,
    >,
) -> impl Responder {
    let result = application
        .restaurant_aggregate
        .handle(&command.into_inner())
        .await;

    match result {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(err) => HttpResponse::InternalServerError().json(json!(err)),
    }
}

#[get("/queries/restaurant")]
async fn get_all_restaurants_handler(
    application: web::Data<
        Application<
            '_,
            AggregateEventRepository,
            AggregateEventRepository,
            OrderViewStateRepository,
            RestaurantViewStateRepository,
        >,
    >,
) -> impl Responder {
    let result = application
        .restaurant_query_handler
        .get_all_restaurants()
        .await;

    match result {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(err) => HttpResponse::InternalServerError().json(json!(err)),
    }
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(restaurant_command_handler)
        .service(order_command_handler)
        .service(get_all_restaurants_handler)
        .service(get_all_orders_handler);
    conf.service(scope);
}
