// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 128]
        password -> Varchar,
        role -> Int2,
        active -> Bool,
        #[max_length = 255]
        notes -> Nullable<Varchar>,
        last_seen -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
