use actix_web_test::server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server()?.await
}
