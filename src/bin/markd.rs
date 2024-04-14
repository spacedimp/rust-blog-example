use std::env;
use sqlx::postgres::{PgPool,PgPoolOptions};
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error>{

    let args: Vec<String> = env::args().collect();
        
    let mut inserter;

    match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            inserter = content;
        },
        Err(error) => {panic!("could not read file")},
    }

    let pool = PgPoolOptions::new()
        .max_connections(3)
        // use your own credentials below
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't create pool");

    let row: (i64,) = sqlx::query_as("insert into myposts (post_title, post_body) values ($1, $2) returning post_id")
        .bind(&args[1])
        .bind(inserter)
        .fetch_one(&pool)
        .await?;

    Ok(())
}
