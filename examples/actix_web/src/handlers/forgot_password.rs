use crate::AppState;
use crate::models::ForgotPasswordRequest;
use actix_web::{HttpResponse, Responder, post, web};
use serde_json::json;

#[post("/forgot-password")]
pub async fn forgot_password(
    state: web::Data<AppState>,
    body: web::Json<ForgotPasswordRequest>,
) -> impl Responder {
    let auth = state.auth.lock().await;

    auth.request_password_reset(&body.email).await;

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Password reset requested"
    }))
}
