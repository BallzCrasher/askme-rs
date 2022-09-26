pub mod models;
pub mod schema;
pub mod commands;

use std::io;
use std::io::Write;
use diesel::prelude::*;
use diesel::insert_into;
use diesel::sql_query;
use models::*;

pub fn print_flush(s: &str) {
    print!("{s}");
    io::stdout().flush().expect("IO Error :: Unable to flush stdout");
}

pub fn clear_screen() {
    print_flush(&format!("{esc}[2J{esc}[1;1H", esc = 27 as char));
}

pub fn user_input<T: std::str::FromStr>(prompt: &str) -> Result<T, <T as std::str::FromStr>::Err>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    print_flush(prompt);
    let mut inp = String::new();
    io::stdin().read_line(&mut inp).expect("IO error");
    return inp.trim().parse::<T>();
}

pub fn establish_connection(url: &str) -> SqliteConnection {
    SqliteConnection::establish(url)
        .expect("Connection Error::Cannot Establish Connection to database")
}

pub fn init_database(conn: &mut SqliteConnection) -> Result<(),diesel::result::Error> {
    sql_query("CREATE TABLE IF NOT EXISTS accounts(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL
            );").execute(conn)?;

    sql_query("CREATE TABLE IF NOT EXISTS questions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                publisher TEXT NOT NULL,
                is_anonymous bool NOT NULL
            );").execute(conn)?;

    Ok(())
}

pub fn signup(conn: &mut SqliteConnection) { 
    use crate::schema::accounts::dsl::*;
    let username: String = user_input("Enter Username: ").unwrap();
    print_flush("Entre Password: ");
    let user_password: String = rpassword::read_password().unwrap();
    let user_password = sha256::digest(user_password);

    insert_into(accounts)
        .values(NewAccount{
            name: username.as_str(),
            password: user_password.as_str()
        })
        .execute(conn)
        .expect("Error adding account to database");
}

pub fn login(conn: &mut SqliteConnection) -> Account {
    use schema::accounts::dsl::*;
    
    let user_id = loop {
        //prompt user for account info
        let username: String = user_input("Username: ").unwrap();
        print_flush("Password: ");
        let user_password: String = rpassword::read_password().unwrap();
        let user_password = sha256::digest(user_password);
        
        //find if the account of this name
        let user_id = accounts
            .filter(name.eq(username))
            .filter(password.eq(user_password))
            .first::<Account>(conn);
        if user_id.is_err() { println!("Invalid Username Or Password."); }
        break user_id.unwrap();
    };

    user_id
} 

pub fn prompt_login(conn: &mut SqliteConnection) -> Account {
    loop {
        println!("Commands: [ 0: signup, 1: login ]");
        let input = user_input::<String>("-~>: ").unwrap();

        match input.to_lowercase().trim() {
            "0" | "signup" => signup(conn),
            "1" | "login" => break login(conn),
            _ => continue
        }
    }
}
