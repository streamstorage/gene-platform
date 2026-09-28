use crate::db::Connection;
use crate::schema::users;
use crate::utils::to_ascii_lowercase;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, prelude::*};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Identifiable, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[serde(deserialize_with = "to_ascii_lowercase")]
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: i32,
    pub active: bool,
    pub notes: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    #[serde(deserialize_with = "to_ascii_lowercase")]
    pub email: String,
    pub password: String,
    pub role: i32,
    #[serde(default)]
    pub active: bool,
    pub notes: Option<String>,
}

impl NewUser {
    pub fn admin(name: &str, email: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_ascii_lowercase(),
            password: password.to_string(),
            role: 2,
            active: true,
            notes: None,
        }
    }
}

#[derive(Deserialize)]
pub struct NewPassword {
    pub current_password: String,
    pub new_password: String,
}

fn encrypt_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

impl User {
    pub async fn login(
        email: &str,
        password: &str,
        conn: &mut Connection,
    ) -> Result<Option<User>, diesel::result::Error> {
        let user = users::table
            .filter(users::dsl::email.eq(email))
            .get_result::<User>(conn)
            .await
            .optional()?;
        if let Some(user) = user
            && let Ok(val) = verify(password, &user.password)
            && val
        {
            return Ok(Some(user));
        }
        Ok(None)
    }

    pub async fn update_last_seen(user_id: i32, conn: &mut Connection) -> QueryResult<usize> {
        diesel::update(users::table.find(user_id))
            .set(users::dsl::last_seen.eq(diesel::dsl::now))
            .execute(conn)
            .await
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

    pub async fn find_user_by_id(user_id: i32, conn: &mut Connection) -> QueryResult<User> {
        users::table.find(user_id).get_result::<User>(conn).await
    }

    pub async fn add_new_user(
        user: NewUser,
        conn: &mut Connection,
    ) -> Result<usize, diesel::result::Error> {
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
            .await
    }

    pub async fn update_password(
        user_id: i32,
        password: NewPassword,
        conn: &mut Connection,
    ) -> Result<bool, diesel::result::Error> {
        let user = users::table.find(user_id).get_result::<User>(conn).await?;
        if let Ok(val) = verify(&password.current_password, &user.password)
            && val
        {
            diesel::update(users::table.find(user.id))
                .set(users::dsl::password.eq(encrypt_password(&password.new_password)))
                .execute(conn)
                .await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn update_user(
        user_id: i32,
        user: NewUser,
        conn: &mut Connection,
    ) -> Result<(), diesel::result::Error> {
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

    pub async fn get_all(conn: &mut Connection) -> QueryResult<Vec<User>> {
        users::table.load::<User>(conn).await
    }

    pub async fn activate(state: bool, ids: Vec<i32>, conn: &mut Connection) -> QueryResult<usize> {
        diesel::update(users::table.filter(users::dsl::id.eq_any(ids)))
            .set(users::dsl::active.eq(state))
            .execute(conn)
            .await
    }
}
