use axum::{http::StatusCode, response::Html, response::IntoResponse};
use axum::{routing::get, Router};
use tokio::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use sqlx::types::time::Date;
use sqlx::FromRow;

use axum::extract::Path;
use axum::extract::State;

use std::sync::Arc;

use askama::Template;

use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub post_title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

#[derive(FromRow, Debug, Clone)]
pub struct Post {
    pub post_title: String,
    pub post_date: Date,
    pub post_body: String,
}

async fn post(
    Path(query_title): Path<String>,
    State(state): State<Arc<Vec<Post>>>,
) -> impl IntoResponse {
    let mut template = PostTemplate {
        post_title: "none",
        post_date: "none".to_string(),
        post_body: "none",
    };

    for i in 0..state.len() {
        if query_title == state[i].post_title {
            template = PostTemplate {
                post_title: &state[i].post_title,
                post_date: state[i].post_date.to_string(),
                post_body: &state[i].post_body,
            };
            break;
        } else {
            continue;
        }
    }

    // 404 if no title found matching the user's query
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    // render the template into HTML and return it to the user
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response(),
    }
}

async fn index(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let s = state.clone();
    let mut plinks: Vec<String> = Vec::new();

    for i in 0..s.len() {
        plinks.push(s[i].post_title.clone());
    }

    let template = IndexTemplate {
        index_title: String::from("My blog"),
        index_links: &plinks,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template. Error {}", err),
        )
            .into_response(),
    }
}

mod filters {

    // This filter removes the dashes that I will be adding in main()
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't connect to the database");

    let mut posts =
        sqlx::query_as::<_, Post>("SELECT post_title, post_date, post_body FROM myposts")
            .fetch_all(&pool)
            .await
            .unwrap();

    for post in &mut posts {
        post.post_title = post.post_title.replace(" ", "-");
    }

    let shared_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/{query_title}", get(post))
        .with_state(shared_state)
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
