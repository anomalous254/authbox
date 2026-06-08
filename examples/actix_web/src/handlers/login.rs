use crate::AppState;
use crate::models::LoginRequest;
use actix_web::{HttpResponse, Responder, post, web};
use serde_json::json;

#[post("/login")]
pub async fn login(state: web::Data<AppState>, body: web::Json<LoginRequest>) -> impl Responder {
    let auth = state.auth.lock().await;

    let result = auth.login(&body.email, &body.password).await;

    match result {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => HttpResponse::Unauthorized().json(json!({
            "error": format!("{:?}", e)
        })),
    }
}
