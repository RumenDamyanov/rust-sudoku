//! REST router for puzzle generation and solving.
//!
//! Enabled with the `server` feature. The `sudoku-server` binary binds this
//! router to `PORT` (default 8080).

use axum::Json;
use axum::Router;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::Deserialize;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{Board, Difficulty, Grid, MAX_GRID_SIZE};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Router used by `sudoku-server` and the server tests.
pub fn create_router() -> Router {
    Router::new()
        .route("/healthz", get(handle_health))
        .route("/health", get(handle_health))
        .route("/generate", post(handle_generate))
        .route("/solve", post(handle_solve))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
}

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": VERSION,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateRequest {
    #[serde(default)]
    difficulty: Option<String>,
    #[serde(default)]
    include_solution: bool,
    #[serde(default)]
    size: Option<usize>,
    #[serde(default)]
    r#box: Option<String>,
    #[serde(default)]
    attempts: Option<usize>,
}

async fn handle_generate(
    Json(req): Json<GenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let d: Difficulty = match req.difficulty.as_deref().unwrap_or("easy") {
        "easy" | "" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        _ => return Err(err_json(StatusCode::BAD_REQUEST, "invalid difficulty")),
    };

    let attempts = req.attempts.unwrap_or(3).max(1);

    if req.size.is_none() && req.r#box.is_none() {
        let puz = Board::generate(d, attempts)
            .map_err(|_| err_json(StatusCode::INTERNAL_SERVER_ERROR, "generation failed"))?;

        let mut res = serde_json::json!({"puzzle": puz});
        if req.include_solution
            && let Some(sol) = puz.solve()
        {
            res["solution"] = serde_json::to_value(sol).unwrap();
        }
        return Ok(Json(res));
    }

    let size = req.size.unwrap_or(0);
    let box_str = req.r#box.as_deref().unwrap_or("");
    if size == 0 || box_str.is_empty() {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "size and box required for variable grid",
        ));
    }
    if size > MAX_GRID_SIZE {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            &format!("grid size {size} exceeds maximum allowed ({MAX_GRID_SIZE})"),
        ));
    }

    let (br, bc) = parse_box_dims(box_str, size)
        .ok_or_else(|| err_json(StatusCode::BAD_REQUEST, "invalid box dims"))?;

    let g = Grid::new(size, br, bc)
        .map_err(|_| err_json(StatusCode::BAD_REQUEST, "invalid grid params"))?;
    let gpuz = g
        .generate(d, attempts)
        .map_err(|_| err_json(StatusCode::INTERNAL_SERVER_ERROR, "generation failed"))?;

    let res = serde_json::json!({
        "size": gpuz.size,
        "boxR": gpuz.box_rows,
        "boxC": gpuz.box_cols,
        "puzzle": gpuz.cells,
    });
    Ok(Json(res))
}

#[derive(Deserialize)]
struct SolveRequest {
    puzzle: Option<Board>,
    string: Option<String>,
}

async fn handle_solve(
    Json(req): Json<SolveRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let board = if let Some(p) = req.puzzle {
        p.validate()
            .map_err(|_| err_json(StatusCode::BAD_REQUEST, "invalid puzzle"))?;
        p
    } else if let Some(ref s) = req.string {
        s.parse::<Board>()
            .map_err(|_| err_json(StatusCode::BAD_REQUEST, "invalid puzzle string"))?
    } else {
        return Err(err_json(StatusCode::BAD_REQUEST, "missing puzzle"));
    };

    match board.solve() {
        Some(sol) => Ok(Json(serde_json::json!({"solution": sol}))),
        None => Err(err_json(StatusCode::UNPROCESSABLE_ENTITY, "unsolvable")),
    }
}

fn err_json(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"error": msg})))
}

fn parse_box_dims(s: &str, size: usize) -> Option<(usize, usize)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return None;
    }
    let br: usize = parts[0].parse().ok()?;
    let bc: usize = parts[1].parse().ok()?;
    if br * bc != size {
        return None;
    }
    Some((br, bc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_healthz() {
        let app = create_router();
        let resp = app
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], VERSION);
    }

    #[tokio::test]
    async fn test_generate_api() {
        let app = create_router();
        let body = serde_json::json!({"difficulty": "medium", "includeSolution": true});
        let resp = app
            .oneshot(
                Request::post("/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_generate_api_with_solution() {
        let app = create_router();
        let body = serde_json::json!({"difficulty": "easy", "includeSolution": true});
        let resp = app
            .oneshot(
                Request::post("/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_generate_api_errors() {
        let app = create_router();
        let resp = app
            .clone()
            .oneshot(Request::get("/generate").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);

        let body = serde_json::json!({"difficulty": "impossible"});
        let resp = app
            .oneshot(
                Request::post("/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_solve_api() {
        let app = create_router();
        let s = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let body = serde_json::json!({"string": s});
        let resp = app
            .oneshot(
                Request::post("/solve")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_solve_api_errors() {
        let app = create_router();
        let resp = app
            .clone()
            .oneshot(Request::get("/solve").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);

        let app2 = create_router();
        let resp = app2
            .oneshot(
                Request::post("/solve")
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let app3 = create_router();
        let resp = app3
            .oneshot(
                Request::post("/solve")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let app4 = create_router();
        let resp = app4
            .oneshot(
                Request::post("/solve")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"string":"xxx"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
