use actix_web::{App, HttpServer, dev::Server, middleware::from_fn, web};

mod auth_middleware;
mod common;
mod handlers;
mod models;
use auth_middleware::auth_middleware;
use common::*;
use handlers::{
    forgot_password::forgot_password, index::index, login::login, me::me, refresh_token::refresh,
    register::register, reset_password::reset_password, verify_email::verify_email,
};

pub struct AppState {
    pub auth: tokio::sync::Mutex<TestAuthService>,
}

pub fn server() -> Result<Server, std::io::Error> {
    println!("Server running at http://127.0.0.1:8080");

    let state = web::Data::new(AppState {
        auth: tokio::sync::Mutex::new(build_test_auth()),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index)
            .service(register)
            .service(login)
            .service(refresh)
            .service(verify_email)
            .service(forgot_password)
            .service(reset_password)
            .service(
                web::scope("/api")
                    .wrap(from_fn(auth_middleware))
                    .service(me),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    Ok(server)
}
