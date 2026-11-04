mod board;
mod game;
mod ws;

use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::board::BoardDefinition;
use crate::ws::{handle_ws_connection, start_background_maintenance, AppState};

#[derive(Debug, Deserialize)]
struct BoardQuery {
    size: Option<u8>,
    mystery: Option<bool>,
}

async fn health_check_api(State(state): State<AppState>) -> impl IntoResponse {
    let rooms_count = state.rooms.read().await.len();
    let sockets_guard = state.sockets.read().await;
    let connections_count: usize = sockets_guard.values().map(|r| r.len()).sum();

    Json(serde_json::json!({
        "status": "ok",
        "service": "calabozos-y-dragones-rust",
        "active_rooms": rooms_count,
        "active_connections": connections_count
    }))
}

async fn get_board_api(Query(params): Query<BoardQuery>) -> impl IntoResponse {
    let size = params.size.unwrap_or(100);
    let mystery = params.mystery.unwrap_or(false);
    let board = if size == 50 {
        BoardDefinition::express().with_mystery_tiles(mystery)
    } else {
        BoardDefinition::classic().with_mystery_tiles(mystery)
    };
    let mut coords = Vec::new();
    for t in 0..=board.total_tiles {
        coords.push(BoardDefinition::tile_coords_for(t, board.total_tiles));
    }

    Json(serde_json::json!({
        "board": board,
        "coordinates": coords
    }))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check_api))
        .route("/healthz", get(health_check_api))
        .route("/api/health", get(health_check_api))
        .route("/api/board", get(get_board_api))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("public"))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::PRAGMA,
            HeaderValue::from_static("no-cache"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::EXPIRES,
            HeaderValue::from_static("0"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("SAMEORIGIN"),
        ))
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let state = AppState::new();

    // Spawn background maintenance task (room reaper & AFK turn timeout)
    tokio::spawn(start_background_maintenance(state.clone()));

    let app = create_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("\n========================================================");
    println!("🐉 CALABOZOS Y DRAGONES ONLINE (RUST / AXUM) 🐉");
    println!("========================================================");
    println!("> Local:   http://localhost:{}", port);
    println!("> Network: http://0.0.0.0:{}", port);
    println!("========================================================\n");

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check_api_direct() {
        let state = AppState::new();
        let res = health_check_api(State(state)).await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_board_api_direct() {
        let res = get_board_api(Query(BoardQuery {
            size: None,
            mystery: None,
        }))
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);

        let res_express = get_board_api(Query(BoardQuery {
            size: Some(50),
            mystery: Some(true),
        }))
        .await
        .into_response();
        assert_eq!(res_express.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_routes_with_service_oneshot() {
        let state = AppState::new();
        let app = create_app(state);

        // 1. /health
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
        assert_eq!(res.headers().get("x-frame-options").unwrap(), "SAMEORIGIN");

        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "calabozos-y-dragones-rust");

        // 2. /api/board
        let req_board = Request::builder()
            .uri("/api/board")
            .body(Body::empty())
            .unwrap();
        let res_board = app.clone().oneshot(req_board).await.unwrap();
        assert_eq!(res_board.status(), StatusCode::OK);

        let board_bytes = axum::body::to_bytes(res_board.into_body(), usize::MAX)
            .await
            .unwrap();
        let board_json: serde_json::Value = serde_json::from_slice(&board_bytes).unwrap();
        assert_eq!(board_json["board"]["total_tiles"], 100);
        assert_eq!(board_json["coordinates"].as_array().unwrap().len(), 101);
    }
}
