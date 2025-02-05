use argon2::{password_hash::{rand_core::OsRng, SaltString}, PasswordHasher, PasswordVerifier, PasswordHash};
use diesel::{pg::PgConnection, prelude::*, insert_into, BelongingToDsl};
use dotenvy::dotenv;
use std::env;
pub mod models;
pub mod schema;
use uuid::Uuid;
use crate::models::{Post, User, NewUser, PostType};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

pub fn create_account(conn: &mut PgConnection, uname: String, password: String) -> Option<User> {
    use schema::users::dsl::*;

    let salt=SaltString::generate(OsRng);
    let argon2=argon2::Argon2::default();
    let hsh=argon2.hash_password(password.as_bytes(), &salt).unwrap();
    
    if users.filter(username.eq(uname.clone())).limit(1).get_result::<(i32, String, String, uuid::Uuid)>(conn).is_ok() { return None; }
    let new_user = NewUser { username: uname, hash: hsh.to_string() };

    Some(diesel::insert_into(users)
	.values(&new_user)
	.returning(User::as_returning())
	.get_result(conn)
	.expect("Error signing up"))
}

pub fn verify_password(conn: &mut PgConnection, uname: String, password: String) -> bool {
    use schema::users::dsl::*;
    
    let hsh: Vec<String> = users
	.filter(username.eq(uname))
	.select(hash)
	.load(conn)
	.expect("Error getting password hash");
    if hsh.is_empty() { return false };
    let argon2=argon2::Argon2::default();
    let hsh_argn=PasswordHash::new(hsh[0].as_str());
    argon2.verify_password(password.as_bytes(), &hsh_argn.unwrap()).is_ok()
}

fn delete_posts_for_user(conn: &mut PgConnection, user: &User) {
    let _ = diesel::delete(Post::belonging_to(user)).execute(conn);
}

pub fn delete_account(conn: &mut PgConnection, uname: String, password: String) {
    use schema::users::dsl::*;

    if verify_password(conn, uname.clone(), password) {
	let user_record = users.filter(username.eq(uname.clone()));
	let user: User = users.filter(username.eq(uname)).first::<User>(conn).expect("Error fetching user");
	delete_posts_for_user(conn, &user);
	diesel::delete(user_record)
	    .execute(conn)
	    .expect("Error deleting account");
    }
}

pub fn new_session(conn: &mut PgConnection, uname: String, password: String) -> (Uuid, bool) {
    use schema::users::dsl::*;
    let mut correct = false;
    let seshn = Uuid::new_v4();
    if verify_password(conn, uname.clone(), password) {
	correct = true;
	let _ = diesel::update(users).filter(username.eq(uname)).set(session_id.eq(seshn)).execute(conn);
    }
    (seshn, correct)
}

pub fn create_post(conn: &mut PgConnection, ttl: String, pst_typ: PostType, cntnt: String, seshn_id: Uuid) -> Post {
    use schema::users::dsl::*;
    use schema::posts::dsl::*;

    let usr_id: i32 = users
	.filter(session_id.eq(seshn_id))
	.select(schema::users::dsl::id)
	.first(conn)
	.expect("Error getting user_id");

    insert_into(posts)
	.values((user_id.eq(usr_id), title.eq(ttl), post_type.eq(pst_typ), content.eq(cntnt)))
	.returning(Post::as_returning())
	.get_result(conn)
	.expect("Error creating post")
}

pub fn get_posts(conn: &mut PgConnection) -> Vec<Post> {
    use schema::posts::dsl::*;
    
    posts
        .select(Post::as_select())
        .load(conn)
        .expect("Error loading posts")
}

pub fn get_user_from_session(conn: &mut PgConnection, sesh: Uuid) -> User {
    use schema::users::dsl::*;

    users
	.filter(session_id.eq(sesh))
	.first(conn)
	.expect("Error loading user")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_creation_verification_deletion() {
	let conn = &mut establish_connection();
	
	dbg!(create_account(conn, "username".to_string(), "password123".to_string()).unwrap_or_else(|| panic!("Username already exists.")));
	assert!(!verify_password(conn, "username".to_string(), "password125".to_string()), "incorrect password allowed");
	assert!(verify_password(conn, "username".to_string(), "password123".to_string()), "correct password dissalowed");
	delete_account(conn, "username".to_string(), "password123".to_string());
    }
    #[test]
    fn post_creation() {
	let conn = &mut establish_connection();

	let account = create_account(conn, "username".to_string(), "password123".to_string()).unwrap_or_else(|| panic!("Username already exists."));
	create_post(conn, "Hello, world".to_string(), PostType::Text, "Hello, world!".to_string(), account.session_id);
	delete_account(conn, "username".to_string(), "password123".to_string());
    }
}
