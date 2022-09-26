use askme_diesel_rs::*;
use commands::*;

fn main() {

    let connection = &mut establish_connection("database.db");
    init_database(connection).expect("Error Initializing database");

    let user_account = prompt_login(connection);
    println!("Signed As: {}", user_account.name);

    loop {
        println!("Commands: [ 1: questions, 2: ask, 3: open_thread, 4: users, 9: clear, 0: exit ]");
        let input: String = user_input("Command: ").unwrap();

        match input.to_lowercase().trim() { 
            "1" | "questions" => get_questions(connection),
            "2" | "ask" => ask_question(connection, &user_account) ,
            "3" | "open_thread" => (),
            "4" | "users" => get_users(connection),
            "9" | "clear" => (),
            "0" | "exit" => break,
            _ => continue
        }
        
    };
}
