use super::auth::get_user_id;
use crate::db::Pool;
use crate::error::ServiceError;
use crate::models::user::{NewUser, User};
use actix_session::Session;
use actix_web::{
    HttpResponse, Result,
    web::{Data, Json, Path},
};
use serde_json::Value;

pub async fn get_profile(pool: Data<Pool>, session: Session) -> Result<HttpResponse> {
    let user_id = get_user_id(&session);
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    let user = User::find_user_by_id(user_id, &mut conn).await?;
    let _ = User::update_last_seen(user_id, &mut conn).await;
    Ok(HttpResponse::Ok().json(user))
}

pub async fn update_profile(
    Json(value): Json<Value>,
    pool: Data<Pool>,
    session: Session,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&session);
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    let new_password = value
        .as_object()
        .and_then(|obj| obj.get("newPassword"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServiceError::bad_request("Field newPassword is missing in the payload"))?
        .to_string();
    let user: User = serde_json::from_value(value).map_err(ServiceError::bad_request)?;

    User::update_with_verification(user_id, user, new_password, &mut conn).await?;
    Ok(HttpResponse::Ok().body("profile updated"))
}

pub async fn list_users(pool: Data<Pool>) -> Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    let users = User::get_all(&mut conn).await?;
    Ok(HttpResponse::Ok().json(users))
}

pub async fn add_user(Json(user): Json<NewUser>, pool: Data<Pool>) -> Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    User::add(user, &mut conn).await?;
    Ok(HttpResponse::Ok().body("user added"))
}

pub async fn update_user(
    user_id: Path<i32>,
    Json(user): Json<User>,
    pool: Data<Pool>,
) -> Result<HttpResponse> {
    let user_id = user_id.into_inner();
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    User::update(user_id, user, &mut conn).await?;
    Ok(HttpResponse::Ok().body("user updated"))
}

pub async fn activate_users(Json(ids): Json<Vec<i32>>, pool: Data<Pool>) -> Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    User::set_state(true, ids, &mut conn).await?;
    Ok(HttpResponse::Ok().body("users activated"))
}

pub async fn deactivate_users(Json(ids): Json<Vec<i32>>, pool: Data<Pool>) -> Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    User::set_state(false, ids, &mut conn).await?;
    Ok(HttpResponse::Ok().body("users deactivated"))
}
