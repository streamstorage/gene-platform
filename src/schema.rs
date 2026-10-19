// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 128]
        password -> Varchar,
        role -> Integer,
        active -> Bool,
        #[max_length = 255]
        notes -> Nullable<Varchar>,
        last_seen -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
