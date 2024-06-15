use axum::{extract::State, http::StatusCode, response::{Html, Redirect}, routing::{get, post}, Router};
use database::{manage_db::get_database, models::User};
use lazy_static::lazy_static;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_sessions::Expiry;

pub mod assets;
pub mod view;
pub mod database;
lazy_static! {
    static ref HOMEPAGE_RELOAD_TIME: u32 = std::env::var("HOMEPAGE_RELOAD_TIME").unwrap_or("10".to_string()).parse::<u32>().unwrap();
    
}

static USER_SESSION_DATA : &str = "USER";
static USER_SESSION_PERMISSION_DATA : &str = "PERMISSIONS";

async fn post_login(State(db_pool): State<sqlx::SqlitePool>, session: tower_sessions::Session, axum::extract::Form(input): axum::extract::Form<InputLoginForm>,) -> Html<String>{
    let user =  User::login(db_pool, &input.username, &input.password).await;
    if let None = user {
        return Html("Errore autenticazione".into());
    }
    let user: User = user.unwrap();
    session.clear().await;
    let user_json = serde_json::to_string(&user).unwrap();
    session.insert(USER_SESSION_DATA, user_json).await.unwrap();    
    
    Html("Autenticazione effettuata con successo".to_string())
}



#[tokio::main]
async fn main() {

    let mut hbs = handlebars::Handlebars::new();
    hbs.register_partial("components/font.hbs", view::COMPONENTS_FONT_HBS).unwrap();
    hbs.register_partial("components/head_dashboard.hbs", view::COMPONENTS_HEAD_DASHBOARD_HBS).unwrap();
    hbs.register_partial("components/style_dashboard.hbs", view::COMPONENTS_STYLE_DASHBOARD_HBS).unwrap();
    hbs.register_partial("components/header.hbs", view::COMPONENTS_HEADER_HBS).unwrap();
    hbs.register_template_string("index.hbs",view::INDEX_HBS).unwrap();
    hbs.register_template_string("dashboard.hbs",view::DASHBOARD_HBS).unwrap();
    hbs.register_template_string("login.hbs",view::LOGIN_HBS).unwrap();

    let db_pool : SqlitePool= get_database().await;
    let session_store = database::manage_db::get_session_database().await;
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
    .with_secure(false)
    .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::hours(2)));

    let app = Router::new()
    .route("/",get(Html(hbs.render("index.hbs", &serde_json::json!({
        "internalDoorUnLocked": true,
        "homepage_reload_time": *HOMEPAGE_RELOAD_TIME
    })).unwrap())))
    //redirect /admin to /admin/dashboard
    .route("/admin",get(Redirect::to("/accounts/admin")))
    .route("/accounts/login", get(Html(hbs.render("login.hbs", &()).unwrap())))
    .route("/accounts/login", post(post_login))
    .fallback((StatusCode::NOT_FOUND, "Not Found"))
    .with_state(db_pool)
    .layer(session_layer);

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();


}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InputLoginForm {
    username: String,
    password: String
}

