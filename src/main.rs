use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use antirev::*;

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("<h1>Hello, world!</h1>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = &mut establish_connection();
    create_account(conn, "goren".to_string(), "password".to_string());
    HttpServer::new(|| {
        App::new()
            .service(home)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
