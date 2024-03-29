use std::alloc::System;
use actix_web::{get, post, web, cookie, HttpResponse};
use tera::{Tera, Context};
use lazy_static::lazy_static;

use serde::{Serialize, Deserialize};
use jsonwebtoken;
use sha2::{Sha512, Digest};

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

mod model;
pub use crate::model::User;


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i64,
}

pub struct AppState {
    db: MySqlPool,
}

const SECRET_KEY: &str = "SOME SECRET KEY";
const PASSWORD_HASH_SALT: &str = "polina loh";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

#[global_allocator]
static A: System = System;

#[get("/home")]
async fn home() -> HttpResponse {
    HttpResponse::Ok().body(
        TEMPLATES.render("home.html", &Context::new()).unwrap()
    )
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body(
        TEMPLATES.render("index.html", &Context::new()).unwrap()
    )
}

#[get("/catalog")]
async fn catalog() -> HttpResponse {
    HttpResponse::Ok().body(
        TEMPLATES.render("catalog.html", &Context::new()).unwrap()
    )
}

#[get("/login")]
async fn get_login() -> HttpResponse {
    let mut ctx = Context::new();

    ctx.insert("signin", &true);

    HttpResponse::Ok().body(
        TEMPLATES.render("login.html", &ctx).unwrap()
    )
}

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[post("/login")]
async fn post_login(data: web::Data<AppState>, form: web::Form<LoginForm>) -> HttpResponse {
    let mut hasher = Sha512::new();
    hasher.update(PASSWORD_HASH_SALT);
    hasher.update(form.password.clone());

    let users: Vec<User> = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE email = ? AND password = ?"#,
        form.email,
        format!("{:x}", hasher.finalize())
    )
    .fetch_all(&data.db)
    .await
    .unwrap();

    if users.len() != 1 {
        return HttpResponse::BadRequest().body("Login failed");
    }

    let claims = &Claims{
        user_id: users[0].id,
    };

    let token: String = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(SECRET_KEY.as_bytes()),
    ).unwrap();

    HttpResponse::Ok()
        .cookie(cookie::Cookie::build("token", token).finish())
        .body(
            TEMPLATES.render("index.html", &Context::new()).unwrap()
        )
}

#[get("/cart")]
async fn cart() -> HttpResponse {
    HttpResponse::Ok().body(
        TEMPLATES.render("catalog.html", &Context::new()).unwrap()
    )
}

#[get("/connect")]
async fn connect() -> HttpResponse {
    HttpResponse::Ok().body(
        TEMPLATES.render("connect.html", &Context::new()).unwrap()
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    use actix_web::{App, HttpServer};
    use std::thread::available_parallelism;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .service(home)
            .service(index)
            .service(catalog)
            .service(get_login)
            .service(post_login)
            .service(cart)
            .service(connect)
    }).workers(available_parallelism().unwrap().get())
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
