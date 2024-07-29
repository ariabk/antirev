// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "post_type"))]
    pub struct PostType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostType;

    posts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        post_type -> PostType,
        content -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hash -> Varchar,
        session_id -> Uuid,
    }
}

diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
