use crate::db::Connection;
use crate::error::{ServiceError, ServiceResult};
use crate::schema::users;
use crate::utils::to_ascii_lowercase;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, prelude::*};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Identifiable, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[serde(deserialize_with = "to_ascii_lowercase")]
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: i16,
    pub active: bool,
    pub notes: Option<String>,
    #[serde(skip_deserializing)]
    pub last_seen: Option<NaiveDateTime>,
    #[serde(skip_deserializing)]
    pub created_at: NaiveDateTime,
    #[serde(skip_deserializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    #[serde(deserialize_with = "to_ascii_lowercase")]
    pub email: String,
    pub password: String,
    pub role: i16,
    #[serde(default)]
    pub active: bool,
    pub notes: Option<String>,
}

impl NewUser {
    pub fn admin(name: &str, email: &str, password: &str) -> Self {
        Self {
            name: name.trim().to_string(),
            email: email.trim().to_ascii_lowercase(),
            password: password.trim().to_string(),
            role: 2,
            active: true,
            notes: None,
        }
    }
}

fn encrypt_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

impl User {
    pub async fn login(
        email: &str,
        password: &str,
        conn: &mut Connection,
    ) -> ServiceResult<Option<User>> {
        let user = users::table
            .filter(users::dsl::email.eq(email))
            .get_result::<User>(conn)
            .await
            .optional()?;
        if let Some(user) = user
            && let Ok(verified) = verify(password, &user.password)
            && verified
        {
            return Ok(Some(user));
        }
        Ok(None)
    }

    pub async fn update_last_seen(user_id: i32, conn: &mut Connection) -> ServiceResult<()> {
        diesel::update(users::table.find(user_id))
            .set(users::dsl::last_seen.eq(diesel::dsl::now))
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn is_active(user_id: i32, conn: &mut Connection) -> bool {
        users::table
            .find(user_id)
            .get_result::<User>(conn)
            .await
            .map(|user| user.active)
            .unwrap_or(false)
    }

    pub async fn is_active_admin(user_id: i32, conn: &mut Connection) -> bool {
        users::table
            .find(user_id)
            .get_result::<User>(conn)
            .await
            .map(|user| user.role == 2 && user.active)
            .unwrap_or(false)
    }

    pub async fn find_user_by_id(user_id: i32, conn: &mut Connection) -> ServiceResult<User> {
        let user = users::table.find(user_id).get_result::<User>(conn).await?;
        Ok(user)
    }

    pub async fn update_with_verification(
        user_id: i32,
        user: User,
        new_password: String,
        conn: &mut Connection,
    ) -> ServiceResult<()> {
        let u = users::table.find(user_id).get_result::<User>(conn).await?;
        if let Ok(verified) = verify(&user.password, &u.password)
            && verified
        {
            if new_password.is_empty() {
                diesel::update(users::table.find(user_id))
                    .set((
                        users::dsl::name.eq(user.name),
                        users::dsl::email.eq(user.email),
                        users::dsl::notes.eq(user.notes),
                    ))
                    .execute(conn)
                    .await?;
            } else {
                diesel::update(users::table.find(user_id))
                    .set((
                        users::dsl::name.eq(user.name),
                        users::dsl::email.eq(user.email),
                        users::dsl::password.eq(encrypt_password(&new_password)),
                        users::dsl::notes.eq(user.notes),
                    ))
                    .execute(conn)
                    .await?;
            }
            Ok(())
        } else {
            Err(ServiceError::bad_request("invalid password"))
        }
    }

    pub async fn get_all(conn: &mut Connection) -> ServiceResult<Vec<User>> {
        let users = users::table
            .order(users::dsl::id.asc())
            .load::<User>(conn)
            .await?;
        Ok(users)
    }

    pub async fn add(user: NewUser, conn: &mut Connection) -> ServiceResult<()> {
        let new_user = NewUser {
            email: user.email,
            name: user.name,
            password: encrypt_password(&user.password),
            role: user.role,
            active: true,
            notes: user.notes,
        };
        diesel::insert_into(users::table)
            .values(new_user)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn update(user_id: i32, user: User, conn: &mut Connection) -> ServiceResult<()> {
        if user.password.is_empty() {
            diesel::update(users::table.find(user_id))
                .set((
                    users::dsl::name.eq(user.name),
                    users::dsl::email.eq(user.email),
                    users::dsl::role.eq(user.role),
                    users::dsl::notes.eq(user.notes),
                ))
                .execute(conn)
                .await?;
        } else {
            diesel::update(users::table.find(user_id))
                .set((
                    users::dsl::name.eq(user.name),
                    users::dsl::email.eq(user.email),
                    users::dsl::password.eq(encrypt_password(&user.password)),
                    users::dsl::role.eq(user.role),
                    users::dsl::notes.eq(user.notes),
                ))
                .execute(conn)
                .await?;
        }
        Ok(())
    }

    pub async fn set_state(state: bool, ids: Vec<i32>, conn: &mut Connection) -> ServiceResult<()> {
        diesel::update(users::table.filter(users::dsl::id.eq_any(ids)))
            .set(users::dsl::active.eq(state))
            .execute(conn)
            .await?;
        Ok(())
    }

    // for cli
    pub async fn reset_password(
        email: String,
        password: String,
        conn: &mut Connection,
    ) -> QueryResult<Option<User>> {
        let user = users::table
            .filter(users::dsl::email.eq(email))
            .get_result::<User>(conn)
            .await
            .optional()?;
        if let Some(user) = user {
            diesel::update(users::table.find(user.id))
                .set(users::dsl::password.eq(encrypt_password(&password)))
                .execute(conn)
                .await?;
            return Ok(Some(user));
        }
        Ok(None)
    }
}
