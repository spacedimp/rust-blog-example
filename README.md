# Rust Blog Example

Full code for the [blog](https://spacedimp.com/blog/using-rust-axum-postgresql-and-tokio-to-build-a-blog/) post on building a simple blog. 

This is an example for those wanting to build a web app using Rust, Axum, and PostgreSQL. 

### How to run

The blog post basically covers it all. Just make sure the database is running with the fields shown in the blog post. Also insert a post into the database before running the app. 

I created a separate binary to handle inserting a markdown file into the DB. However it's inefficient in the long run as it has now way to edit a blog post. I leave it to the user to create a better way of inserting, updating, deleting blog posts into the database ;)


```
cargo run --bin markd "some title" ./some_post.md
```

then spin up the server

```
cargo run --bin blog-rs
```
