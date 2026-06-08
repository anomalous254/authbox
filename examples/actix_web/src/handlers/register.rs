use crate::AppState;
use crate::common::*;
use actix_web::{HttpResponse, Responder, post, web};
use serde_json::json;

#[post("/register")]
pub async fn register(state: web::Data<AppState>, body: web::Json<RegisterDto>) -> impl Responder {
    let mut auth = state.auth.lock().await;

    let result = auth
        .register(RegisterDto {
            email: body.email.clone(),
            password: body.password.clone(),
            username: body.username.clone(),
            phone: body.phone.clone(),
            country: body.country.clone(),
            city: body.city.clone(),
            age: body.age,
        })
        .await;

    match result {
        Ok(user) => HttpResponse::Created().json(json!({
            "id": user.id,
            "email": user.email,
            "verified": user.is_email_verified
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "error": format!("{:?}", e)
        })),
    }
}
