use crate::{
    kafka::producer::{ProductEvent, ProductEventType},
    products::{
        models::{
            CreateProductRequest, Product, ProductResponse, UpdateProductRequest,
        },
        utils::{products_to_responses, send_kafka_event},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use bson::{Bson, DateTime};
use mongodb::bson::{Document, oid::ObjectId};
use std::str::FromStr;
use tracing::{error, info, warn};


#[utoipa::path(
    post,
    path = "",
    tag = "product",
    responses(
        (status = 201, description = "Create products successfully", body = [ProductResponse])
    ),
    security(
        ("token" = [])
    )
)]
pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let new_product = Product {
        _id: Some(ObjectId::new()),
        name: payload.name,
        description: payload.description,
        price: payload.price,
        created_at: now,
        updated_at: now,
    };

    let product_for_event = new_product.clone();

    match state.db_repo.create_product(new_product).await {
        Ok(inserted_id) => {
            info!("Product created successfully with ID: {}", inserted_id);

            let event = ProductEvent {
                event_type: ProductEventType::Created,
                product_id: inserted_id.to_hex(),
                payload: Some(ProductResponse::from_product(&product_for_event)),
                timestamp: chrono::Utc::now(),
            };
            send_kafka_event(
                &state.kafka_producer,
                &state.config.kafka_product_events_topic,
                event,
            )
            .await;

            let response = ProductResponse::from_product(&product_for_event);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create product in DB: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create product".to_string(),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "product",
    responses(
        (status = 200, description = "Get product successfully", body = [ProductResponse])
    ),
    params(
        ("id" = String, Path, description = "product id")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ProductResponse>), (StatusCode, String)> {
    let object_id = match ObjectId::from_str(&id) {
        Ok(oid) => oid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid product ID format".to_string(),
            ));
        }
    };

    match state.db_repo.find_product_by_id(object_id).await {
        Ok(Some(product)) => {
            info!("Product found: {}", id);
            let response = ProductResponse::from_product(&product);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            warn!("Product not found: {}", id);
            Err((StatusCode::NOT_FOUND, "Product not found".to_string()))
        }
        Err(e) => {
            error!("Failed to fetch product {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve product".to_string(),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "product",
    responses(
        (status = 200, description = "List all products successfully", body = [ProductResponse])
    ),
    security(
        ("token" = [])
    )
)]
pub async fn list_products(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match state.db_repo.find_all_products().await {
        Ok(products) => {
            info!("Retrieved {} products", products.len());
            let response = products_to_responses(&products);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to list products: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "product",
    responses(
        (status = 200, description = "Update products successfully", body = [ProductResponse])
    ),
    params(
        ("id" = String, Path, description = "product id")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let object_id = match ObjectId::from_str(&id) {
        Ok(oid) => oid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid product ID format".to_string(),
            ));
        }
    };

    let mut update_doc = Document::new();
    
    if let Some(name) = payload.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = payload.description {
        update_doc.insert("description", desc);
    }
    if let Some(price) = payload.price {
        update_doc.insert("price", price);
    }

    if update_doc.is_empty() {
        return get_product(State(state), Path(id)).await;
    }

    update_doc.insert("updated_at", Bson::DateTime(DateTime::now()));

    match state.db_repo.update_product(object_id, update_doc).await {
        Ok(true) => {
            info!("Product updated successfully: {}", id);
            match state.db_repo.find_product_by_id(object_id).await {
                Ok(Some(updated_product)) => {
                    let response = ProductResponse::from_product(&updated_product);
                    let event = ProductEvent {
                        event_type: ProductEventType::Updated,
                        product_id: id.clone(),
                        payload: Some(response.clone()),
                        timestamp: chrono::Utc::now(),
                    };
                    send_kafka_event(
                        &state.kafka_producer,
                        &state.config.kafka_product_events_topic,
                        event,
                    )
                    .await;
                    Ok((StatusCode::OK, Json(response)))
                }
                Ok(None) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to retrieve updated product".to_string(),
                )),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to retrieve updated product".to_string(),
                )),
            }
        }
        Ok(false) => {
            warn!("Product not found for update: {}", id);
            Err((StatusCode::NOT_FOUND, "Product not found".to_string()))
        }
        Err(e) => {
            error!("Failed to update product {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update product".to_string(),
            ))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "product",
    responses(
        (status = 200, description = "Delete products successfully", body = [ProductResponse])
    ),
    params(
        ("id" = String, Path, description = "product id")
    ),
    security(
        ("token" = [])
    )
)]
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let object_id = match ObjectId::from_str(&id) {
        Ok(oid) => oid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid product ID format".to_string(),
            ));
        }
    };

    match state.db_repo.delete_product(object_id).await {
        Ok(true) => {
            info!("Product deleted successfully: {}", id);

            let event = ProductEvent::<()> {
                event_type: ProductEventType::Deleted,
                product_id: id.clone(),
                payload: None,
                timestamp: chrono::Utc::now(),
            };
            send_kafka_event(
                &state.kafka_producer,
                &state.config.kafka_product_events_topic,
                event,
            )
            .await;

            Ok((StatusCode::NO_CONTENT, ()))
        }
        Ok(false) => {
            warn!("Product not found for deletion: {}", id);
            Err((StatusCode::NOT_FOUND, "Product not found".to_string()))
        }
        Err(e) => {
            error!("Failed to delete product {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete product".to_string(),
            ))
        }
    }
}
