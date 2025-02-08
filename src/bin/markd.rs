use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // collects the arguments when we run:
    // cargo run --bin markd "A title" ./post.md

    let args: Vec<String> = env::args().collect();

    let mut inserter;

    // argument 2 should contain the file name
    match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            inserter = content;
        }
        Err(error) => {
            panic!("could not insert into postgres")
        }
    }

    let pool = PgPoolOptions::new()
        .max_connections(3)
        // use your own credentials below
        .connect("postgres://myuser:mypass@localhost/mydb")
        .await
        .expect("couldn't create pool");

    // insert the title and file contents into the database
    let row: (i64,) = sqlx::query_as(
        "insert into myposts (post_title, post_body) values ($1, $2) returning post_id",
    )
    .bind(&args[1])
    .bind(inserter)
    .fetch_one(&pool)
    .await?;

    Ok(())
}
