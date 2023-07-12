use clap::{Parser, Subcommand};

use diesel::prelude::*;
use diesel_demo::{models::Post, *};
use std::io::{stdin, Read};

#[derive(Debug, Parser)]
#[command(name = "diesel_demo")]
#[command(about = "Diesel Demo to showcase a post system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Create a new post")]
    Write,
    #[command(about = "Publish a post")]
    Publish {
        #[arg(value_name = "POST_ID", required = true)]
        id: i32,
    },
    #[command(about = "Show published posts")]
    Show,
    #[command(about = "Delete a post")]
    Delete {
        #[arg(value_name = "POST_TITLE", required = true)]
        target: String,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Write => {
            let connection = &mut establish_connection();

            let mut title = String::new();
            let mut body = String::new();

            println!("What would you like your title to be?");
            stdin().read_line(&mut title).unwrap();
            let title = title.trim_end(); // Remove the trailing newline

            println!(
                "\nOk! Let's write {} (Press {} when finished)\n",
                title, EOF
            );
            stdin().read_to_string(&mut body).unwrap();

            let post = create_post(connection, title, &body);
            println!("\nSaved draft {} with id {}", title, post.id);
        }
        Commands::Publish { id } => {
            use self::schema::posts::dsl::{posts, published};

            let connection = &mut establish_connection();

            let post = diesel::update(posts.find(id))
                .set(published.eq(true))
                .returning(Post::as_returning())
                .get_result(connection)
                .unwrap();
            println!("Published post {}", post.title);
        }
        Commands::Show => {
            use self::schema::posts::dsl::*;

            let connection = &mut establish_connection();
            let results = posts
                .filter(published.eq(true))
                .limit(5)
                .select(Post::as_select())
                .load(connection)
                .expect("Error loading posts");

            println!("Displaying {} posts", results.len());
            for post in results {
                println!("{}", post.title);
                println!("-----------\n");
                println!("{}", post.body);
            }
        }
        Commands::Delete { target } => {
            use self::schema::posts::dsl::*;

            let pattern = format!("%{}%", target);

            let connection = &mut establish_connection();
            let num_deleted = diesel::delete(posts.filter(title.like(pattern)))
                .execute(connection)
                .expect("Error deleting posts");

            println!("Deleted {} posts", num_deleted);
        }
    }
}

#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
