use crate::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use authbox::prelude::{JwtClaims, UserStore};

#[get("/me")]
pub async fn me(state: web::Data<AppState>, claims: web::ReqData<JwtClaims>) -> impl Responder {
    let auth = state.auth.lock().await;
    let claims = claims.into_inner();
    let user_id = claims.sub.clone();
    let user = auth.store.find_by_id(&user_id).await.unwrap();
    HttpResponse::Ok().json(user)
}
