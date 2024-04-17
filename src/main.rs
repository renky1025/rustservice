mod sqlite_utilmod;

use axum::{
    extract::Extension,
    extract::Request,
    extract::Query,
    http::StatusCode,
    http::header::{HeaderMap, AUTHORIZATION},
    routing::{get, post, put, delete, any},
    middleware::{self, Next},
    response::Response,
    Json,
    extract::Path,
    Router,
};
use tower_http::add_extension::AddExtensionLayer;
use rusqlite::Connection;
use sqlite_utilmod::sqlite_utilmod::update_person;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use serde_json::{json,Value};
use serde::{Deserialize, Serialize};
use tokio::signal;
use axum::response::IntoResponse;
use log::info;

use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
// the input to our `ResponseDTO` handler
#[derive(Debug, Serialize, Deserialize)]
struct ResponseDTO {
    code: u32,
    message: String,
    data: Value,
}
#[derive(Debug, Serialize, Deserialize)]
struct PageResult {
    current: i32,
    page_size: i32,
    total: i32,
    records: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyRequestType {
    message: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    page_no: usize,
    page_size: usize,
}


async fn route_not_found() -> &'static str { "page not found" }

// async fn logging_middleware(req: Request<Body>, next: Next<Body>) -> Response {
//     println!("Received a request to {}", req.uri());
//     next.run(req).await
// }

async fn auth(
    // run the `HeaderMap` extractor
    headers: HeaderMap,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("headers {}",
        match headers.get(AUTHORIZATION) {
            None => "No middle name!",
            Some(x) => x.to_str().unwrap(),
        }
    );
    let token =  match headers.get(AUTHORIZATION) {
        None => "",
        Some(x) => x.to_str().unwrap(),
    };

    let s = token.trim_start_matches("Bearer ");
    if s.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if token_is_valid(s){
        let response = next.run(request).await;
        Ok(response)
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}


fn token_is_valid(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    true
}
use crate::sqlite_utilmod::sqlite_utilmod::open_my_db;
use crate::sqlite_utilmod::sqlite_utilmod::select_all;
use crate::sqlite_utilmod::sqlite_utilmod::select_one;
use crate::sqlite_utilmod::sqlite_utilmod::PersonDTO;
use crate::sqlite_utilmod::sqlite_utilmod::remove_person;
use crate::sqlite_utilmod::sqlite_utilmod::insert_person;
use crate::sqlite_utilmod::sqlite_utilmod::countperson;


#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    info!("🚀 Server starting...");

    let connection = open_my_db().unwrap();
    let db = Arc::new(Mutex::new(connection));


    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("80".to_string());

    info!("Server configured to accept connections on host {}...", app_host);
    info!("Server configured to listen connections on port {}...", app_port);

    match app_environment.as_str() {
        "development" => {
            info!("Running in development mode");
        }
        "production" => {
            info!("Running in production mode");
        }
        _ => {
            info!("Running in development mode");
        }
    }
   
    // initialize tracing
    //tracing_subscriber::fmt::init();
    // let routes = Router::new()
    // .merge(service_controller::router()
    // .layer(cors));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/status", get(status))

        .route("/personlist", get(queryperson))
        .route("/person/:id", get(getperson_byid))
        .route("/person/:id", put(updateperson_byid))
        .route("/person", post(create_person))
        .route("/person/:id", delete(removeperson_byid))
        .layer(AddExtensionLayer::new(db))
        .fallback(any(route_not_found))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
        //.route_layer(middleware::from_fn(auth));

    // run our app with hyper, listening globally on port 3000
    // let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    // println!("listening on {}", addr);
    let addr = format!("{}:{}", app_host, app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();
}
async fn status() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");

    let response = json!({
        "data": {
            "version": version,
        },
        "message": "Service is running..."
    });
    (StatusCode::OK, Json(response))
}


async fn queryperson(db: Extension<Arc<Mutex<Connection>>>, pagination: Query<Pagination>) -> impl IntoResponse{
    //let connection = open_my_db().unwrap();
    let connection = db.lock().await;
    let res = select_all(&connection, pagination.page_no as i32, pagination.page_size as i32);
    match res {
        Ok(res) => {
            let a = serde_json::json!(res);
            let page = PageResult{
                current: pagination.page_no as i32,
                page_size: pagination.page_size as i32,
                total: countperson(&connection).unwrap() as i32,
                records: a,
            };
            let response = Json(ResponseDTO{ code: 200, message: "ok".to_string(), data: serde_json::json!(page)});
            (StatusCode::OK, response)
        },
        Err(err) => {
            //let response = Json(err);
            let a = ResponseDTO{ code: 500, message: "failed".to_string(), data: serde_json::json!("")};
            println!("Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(a))
        }
    }
}

async fn removeperson_byid(db: Extension<Arc<Mutex<Connection>>>, Path(id): Path<i64>,) -> impl IntoResponse{
    let connection = db.lock().await;
    let res = remove_person(&connection, [id].to_vec());
    match res {
        Ok(res) => {
            let a = serde_json::to_string(&res);
            let response = Json(ResponseDTO{ code: 200, message: "ok".to_string(), data: serde_json::json!(a.unwrap())});
            (StatusCode::OK, response)
        },
        Err(err) => {
            //let response = Json(err);
            let a = ResponseDTO{ code: 500, message: "failed".to_string(), data: serde_json::json!(0)};
            println!("Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(a))
        }
    }
}


async fn getperson_byid(db: Extension<Arc<Mutex<Connection>>>, Path(id): Path<i64>,) -> impl IntoResponse{
    let connection = db.lock().await;
    let res = select_one(&connection, id);
    match res {
        Ok(res) => {
            let response = Json(ResponseDTO{ code: 200, message: "ok".to_string(), data: serde_json::json!(res)});
            (StatusCode::OK, response)
        },
        Err(err) => {
            //let response = Json(err);
            let a = ResponseDTO{ code: 500, message: "failed".to_string(), data: serde_json::json!(0)};
            println!("Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(a))
        }
    }
}

async fn create_person(db: Extension<Arc<Mutex<Connection>>>, Json(payload): Json<PersonDTO>) -> impl IntoResponse{
    let connection = db.lock().await;
    let res = insert_person(&connection, &payload);
    match res {
        Ok(num) => {
            (StatusCode::OK, Json(ResponseDTO{ code: 200, message: "ok".to_string(), data: serde_json::json!(num)}))
        },
        Err(err) => {
            //let response = Json(err);
            let a  = ResponseDTO{ code: 500, message: "failed".to_string(), data: serde_json::json!(0)};
            println!("Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(a))
        }
    }

}

async fn updateperson_byid(db: Extension<Arc<Mutex<Connection>>>, Path(id): Path<i64>,
                            Json(payload): Json<PersonDTO>) -> impl IntoResponse{
    let connection = db.lock().await;
    let res = update_person(&connection, id, &payload);
    match res {
        Ok(res) => {
            let response = Json(ResponseDTO{ code: 200, message: "ok".to_string(), data: serde_json::json!(res)});
            (StatusCode::OK, response)
        },
        Err(err) => {
            //let response = Json(err);
            let a = ResponseDTO{ code: 500, message: "failed".to_string(), data: serde_json::json!(0)};
            println!("Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(a))
        }
    }

}


// async fn handle_error(err: Error) -> (StatusCode, String) {
//     return (
//         StatusCode::INTERNAL_SERVER_ERROR,
//         format!("Something went wrong: {}", err),
//     );
//  }

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}



