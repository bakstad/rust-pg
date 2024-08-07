use actix_web::web::{Json, Path, Query};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::query_dsl::InternalJoinDsl;
use diesel::IntoSql;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name; // <- get app_name
    let body = format!("Hello {app_name}!");

    HttpResponse::Ok().body(body)
}

#[get("/test/{user_id}/{name}")]
async fn test(path: Path<(u32, String)>) -> impl Responder {
    let (user_id, name) = path.into_inner();

    HttpResponse::Ok().body(format!("User: {user_id}, name: {name}"))
}

#[get("/test2/{user_id}/{name}")]
async fn test2(path: Path<TestPathInfo>) -> impl Responder {
    let TestPathInfo { user_id, name } = path.into_inner();

    HttpResponse::Ok().body(format!("-- User: {user_id}, name: {name}"))
}

#[derive(Deserialize)]
struct QueryData {
    user_id: i32,
}

#[post("/post")]
async fn post(body: Json<TestPathInfo>, query: Query<QueryData>) -> impl Responder {
    let QueryData { user_id } = query.into_inner();

    let x = format!("q = {user_id}: {:?}", body);
    println!("{x}");

    HttpResponse::Ok().body(x)
}

#[derive(Debug, Deserialize)]
struct TestPathInfo {
    user_id: u32,
    name: String,
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    app_name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(async {
        let mut i = 0;
        loop {
            println!("Background task: {}", i);
            i += 1;

            sleep(Duration::from_secs(1)).await;
        }
    });

    let app_state = web::Data::new(AppState {
        app_name: String::from("Actix LOLLOL"),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(test)
            .service(test2)
            .service(post)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
