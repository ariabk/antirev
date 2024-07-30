use std::{fs::File, io::BufReader};
use actix_web::{get, post, web::{self, Redirect}, App, HttpRequest, HttpResponse, HttpServer, Responder, cookie::Cookie};
use serde::Deserialize;
use antirev::{*, models::Post, models::PostType};
use rustls::{pki_types::PrivateKeyDer, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use askama_actix::Template;
use uuid::Uuid;

#[derive(Deserialize)]
struct UserForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct PostForm {
    title: String,
    content: String,
    post_type: PostType,
}

#[derive(Template)]
#[template(path="feed.html")]
struct FeedTemplate {
    posts: Vec<Post>,
    signed_in: bool
}

#[post("/signup")]
async fn signup_post(form: web::Form<UserForm>) -> impl Responder {
    let conn = &mut establish_connection();
    create_account(conn, form.username.clone(), form.password.clone());
    let (session, _) = new_session(conn, form.username.clone(), form.password.clone());
    return HttpResponse::SeeOther()
        .append_header(("Location", "/feed"))
        .cookie(Cookie::new("session_id", session.to_string()))
        .finish();
}

#[get("/signup")]
async fn signup_get() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/signup.html"))
}

#[post("/signin")]
async fn signin_post(form: web::Form<UserForm>) -> impl Responder {
    let conn = &mut establish_connection();

    let (session, successful) = new_session(conn, form.username.clone(), form.password.clone());
    if successful {
	return HttpResponse::SeeOther()
            .append_header(("Location", "/feed"))
            .cookie(Cookie::new("session_id", session.to_string()))
            .finish();
    }
    HttpResponse::SeeOther()
        .append_header(("Location", "/signin"))
	.finish()
}

#[get("/signin")]
async fn signin_get() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/signin.html"))
}

#[post("/post")]
async fn post_post(req: HttpRequest, form: web::Form<PostForm>) -> impl Responder {
    let conn = &mut establish_connection();
    let sesh_ = req.cookie("session_id");

    if let Some(sesh) = sesh_ {
	create_post(conn, form.title.clone(), form.post_type.clone(), form.content.clone(), Uuid::parse_str(sesh.value()).expect("Error parsing UUID"));
	return Redirect::to("/feed").see_other();
    }
    Redirect::to("/signin").see_other()
}

#[get("/post")]
async fn post_get() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/post.html"))
}

#[get("/feed")]
async fn feed(req: HttpRequest) -> impl Responder {
    let conn = &mut establish_connection();

    FeedTemplate { posts: get_posts(conn), signed_in: req.cookie("session_id").is_some() }
}

#[post("/signout")]
async fn signout(req: HttpRequest) -> impl Responder {
    if let Some(mut session_id) = req.cookie("session_id") {
	session_id.make_removal();
	return HttpResponse::SeeOther()
	    .append_header(("Location", "/feed"))
	    .cookie(session_id)
	    .finish();
    } else {
	return HttpResponse::BadRequest()
	    .append_header(("Content-Type", "text/plain"))
	    .body("Hey, man. You're not signed in. You can't do that! You can't sign out :,(");
    }
	
}

#[get("/")]
async fn index() -> impl Responder {
    Redirect::to("/feed")
	.permanent()
}

#[get("/output.css")]
async fn style() -> impl Responder {
    HttpResponse::Ok()
	.insert_header(("Content-Type", "text/css"))
	.body(include_str!("../templates/output.css"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = load_rustls_config();
    
    println!("Starting server in https://localhost:8080");
    HttpServer::new(|| {
        App::new()
	    .service(style)
	    .service(signup_post)
	    .service(signup_get)
	    .service(signin_post)
	    .service(signin_get)
	    .service(post_get)
	    .service(post_post)
	    .service(signout)
	    .service(feed)
	    .service(index)
    })
	.bind_rustls_0_23("127.0.0.1:8080", config)?
	.run()
	.await
}

fn load_rustls_config() -> rustls::ServerConfig {
    // stole--erm, borrowed from https://github.com/actix/examples/blob/master/https-tls/rustls/src/main.rs
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let config = ServerConfig::builder().with_no_client_auth();

    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());

    let cert_chain = certs(cert_file).collect::<Result<Vec<_>, _>>().unwrap();
    let mut keys = pkcs8_private_keys(key_file)
        .map(|key| key.map(PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
