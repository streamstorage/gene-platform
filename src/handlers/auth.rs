use crate::db::Pool;
use crate::error::ServiceError;
use crate::models::user::User;
use crate::utils::to_ascii_lowercase;
use actix_session::Session;
use actix_web::{
    HttpResponse, Result,
    web::{Data, Json},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Credential {
    #[serde(deserialize_with = "to_ascii_lowercase")]
    pub email: String,
    pub password: String,
}

pub async fn login(
    Json(cred): Json<Credential>,
    pool: Data<Pool>,
    session: Session,
) -> Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(ServiceError::pool_error)?;
    let user = User::login(&cred.email, &cred.password, &mut conn)
        .await
        .map_err(|e| ServiceError::internal_error_msg(e, "User login failed"))?;
    if let Some(user) = user {
        session
            .insert("user_id", user.id)
            .map_err(|e| ServiceError::internal_error_msg(e, "Session insert failed"))?;

        if User::is_active(user.id, &mut conn).await {
            Ok(HttpResponse::Ok().json(user))
        } else {
            Err(ServiceError::forbidden("User is deactivated").into())
        }
    } else {
        Err(ServiceError::unauthorized("Invalid email or password").into())
    }
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.purge();
    Ok(HttpResponse::Ok().body("signed out"))
}

pub fn get_user_id(session: &Session) -> i32 {
    session.get::<i32>("user_id").unwrap().unwrap()
}
