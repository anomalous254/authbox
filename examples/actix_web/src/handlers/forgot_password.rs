use crate::AppState;
use crate::models::ForgotPasswordRequest;
use actix_web::{HttpResponse, Responder, post, web};

#[post("/forgot-password")]
pub async fn forgot_password(
    state: web::Data<AppState>,
    body: web::Json<ForgotPasswordRequest>,
) -> impl Responder {
    let auth = state.auth.lock().await;

    match auth.request_password_reset(&body.email).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Password reset requested"
        })),

        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("{:?}", e)
        })),
    }
}
