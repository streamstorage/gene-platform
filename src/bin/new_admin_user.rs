use dotenv::dotenv;
use gene_plarform::db::DB;
use gene_plarform::models::user::{NewUser, User};
use std::io::stdin;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut conn = DB::default().init_async_conn().await;

    let mut email = String::new();
    let mut name = String::new();
    let mut password = String::new();

    println!("What is your email?");
    stdin().read_line(&mut email).unwrap();
    let email = email.trim().to_lowercase();

    println!("What is your name?");
    stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    println!("What would you like your password to be?");
    stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    let admin = NewUser::admin(name, &email, password);
    User::add_new_user(admin, &mut conn).await.unwrap();
    println!("\nSaved admin user {} with email {}", name, email);
}
