use dotenv::dotenv;
use gene_platform::db::DB;
use gene_platform::handlers::auth::Credential;
use gene_platform::models::user::User;
use std::io::stdin;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut conn = DB::default().init_async_conn().await;

    let mut email = String::new();
    let mut password = String::new();

    println!("Which email would you like to reset the password for?");
    stdin().read_line(&mut email).unwrap();
    let email = email.trim().to_lowercase();

    println!("What would you like your password to be?");
    stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    let cred = Credential {
        email,
        password: password.to_string(),
    };
    if let Ok(user) = User::reset_password(cred, &mut conn).await
        && let Some(user) = user
    {
        println!("\nPassword reset successfully for user: {}", user.name);
    }
}
