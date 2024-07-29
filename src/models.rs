use diesel::prelude::*;
use crate::schema::*;
use uuid::Uuid;

use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output, IsNull};
use diesel::pg::Pg;
use diesel::sql_types::Integer;
use std::io::Write;
use diesel::{AsExpression, FromSqlRow, QueryId};
use diesel::backend::Backend;
use diesel::query_builder::{QueryFragment, AstPass};
use diesel_derive_enum::DbEnum;
use diesel::SqlType;

#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::PostType"]
pub enum PostType {
    Url,
    Text,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hash: String,
    pub session_id: Uuid
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub hash: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub post_type: PostType,
    pub content: String
}

