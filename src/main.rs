use axum::{
    http::StatusCode, routing::{get, Router},
    response::{Html, IntoResponse},
    extract::{State, Path},
};

use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use sqlx::types::time::Date;

use tower_http::services::ServeDir;

use std::sync::Arc;

use askama::Template;

// post template 
// localhost:4000/post/:query_title
#[derive(Template)]
#[template(path = "posts.html")]
struct PostTemplate<'a> {
    post_title: &'a str,
    post_date: String,
    post_body: &'a str,
}

// homepage template 
// localhost:4000/ 
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

// SQL query will return all posts  
// into a Vec<Post>
#[derive(FromRow,Debug, Clone)]
pub struct Post {
    pub post_title: String,
    pub post_date: Date,
    pub post_body: String,
}

// Our custom Askama filter to replace spaces with dashes in the title
mod filters {

    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
     }
}

// post router uses two extractors 
// Path to extract the query: localhost:4000/post/thispart
// State that holds a Vec<Post> used to render the post that the query matches 
async fn post(Path(query_title): Path<String>, State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let mut template = PostTemplate{post_title: "none", post_date: "none".to_string(), post_body: "none"};
    // if the user's query matches a post title then render a template 
    for i in 0..state.len() {
        if query_title == state[i].post_title {
            template = PostTemplate{post_title: &state[i].post_title, 
                       post_date: state[i].post_date.to_string(), 
                       post_body: &state[i].post_body
            };
            break;
        } else {
            continue
        }
    }

    // 404 if no title found matching the user's query 
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response()
    }
}

// index router (homepage) will return all blog titles in anchor links 
async fn index(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse{

    let s = state.clone();
    let mut plinks: Vec<String> = Vec::new();

    for i in 0 .. s.len() {
        plinks.push(s[i].post_title.clone());
    }

    let template = IndexTemplate{index_title: String::from("My blog"), index_links: &plinks};

    match template.render() {
            Ok(html) => Html(html).into_response(),
         Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error {}", err),
            ).into_response(),
    }
}

#[tokio::main]
async fn main() {

    let pool = PgPoolOptions::new()
                .max_connections(5)
                // replace with your own credentials
                .connect("postgres://myuser:mypass@localhost/mydb")
                .await
                .expect("couldn't connect to the database");

    let mut posts = sqlx::query_as::<_, Post>("select post_title, post_date, post_body from myposts") 
        .fetch_all(&pool)
        .await
        .unwrap();

    for post in &mut posts {
       post.post_title = post.post_title.replace(" ", "-");
    }

    let shared_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(index))
        .route("/post/:query_title", get(post))
        .with_state(shared_state)
        .nest_service("/assets", ServeDir::new("assets"));

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
