use argon2::{password_hash::{rand_core::OsRng, SaltString}, PasswordHasher, PasswordVerifier, PasswordHash};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
pub mod models;
pub mod schema;
use uuid::Uuid;
use crate::models::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_account(conn: &mut PgConnection, uname: String, password: String) -> Option<User> {
    use crate::schema::users;
    use self::schema::users::dsl::*;

    let salt=SaltString::generate(OsRng);
    let argon2=argon2::Argon2::default();
    let hsh=argon2.hash_password(password.as_bytes(), &salt).unwrap();
    
    if users.filter(username.eq(uname.clone())).limit(1).get_result::<(i32, String, String, uuid::Uuid)>(conn).is_ok() { return None; }
    let new_user = NewUser { username: uname, hash: hsh.to_string() };

    Some(diesel::insert_into(users::table)
	.values(&new_user)
	.returning(User::as_returning())
	.get_result(conn)
	.expect("Error signing up"))
}

pub fn verify_password(conn: &mut PgConnection, uname: String, password: String) -> bool {
    use self::schema::users::dsl::*;
    
    let user=users.filter(username.eq(uname));
    let hsh: Vec<String> = user
	.select(hash)
	.load(conn)
	.expect("Error getting password hash");
    if hsh.is_empty() { return false };
    let argon2=argon2::Argon2::default();
    let hsh_argn=PasswordHash::new(hsh[0].as_str());
    argon2.verify_password(password.as_bytes(), &hsh_argn.unwrap()).is_ok()
}

pub fn delete_account(conn: &mut PgConnection, uname: String, password: String) {
    use self::schema::users::dsl::*;

    if verify_password(conn, uname.clone(), password) {
	diesel::delete(users.filter(username.eq(uname)))
	    .execute(conn)
	    .expect("Error deleting account");
    }
}

pub fn new_session(conn: &mut PgConnection, uname: String, password: String) {
    use schema::users::dsl::*;
    
    if verify_password(conn, uname.clone(), password) {
	let _ = diesel::update(users).filter(username.eq(uname)).set(session_id.eq(Uuid::new_v4())).execute(conn);
    }
}

pub fn create_post(conn: &mut PgConnection, title: String, post_type: PostType, content: String, session_id: Uuid) -> /* Post */ () {
    use crate::schema::posts;
    use schema::users::dsl::*;

    
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_creation_verification_deletion() {
	let conn = &mut establish_connection();
	
	dbg!(create_account(conn, "username".to_string(), "password123".to_string()).unwrap());
	assert!(!verify_password(conn, "username".to_string(), "password125".to_string()), "incorrect password allowed");
	assert!(verify_password(conn, "username".to_string(), "password123".to_string()), "correct password dissalowed");
	delete_account(conn, "username".to_string(), "password123".to_string());
    }
}
