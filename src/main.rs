use askme_diesel_rs::*;
use commands::*;

fn main() {
    let connection = &mut establish_connection("database.db");
    init_database(connection).expect("Error Initializing database");

    let user_account = prompt_login(connection);
    println!("Signed As: {}", user_account.name);

    loop {
        println!("Commands: [ 1: questions, 2: ask, 3: ask anonymous, 4: open thread, 5: users, 6: my answers, 7: delete thread, 8: delete account, 9: clear, 0: exit ]");
        let input: String = user_input("Command: ").unwrap();

        match input.to_lowercase().trim() { 
            "1" | "questions" => get_questions(connection),
            "2" | "ask" => ask_question(connection, &user_account, false),
            "3" | "ask anonymous" => ask_question(connection, &user_account, true),
            "4" | "open thread" => open_thread(connection, &user_account),
            "5" | "users" => get_users(connection),
            "6" | "my answers" => user_answers(connection, &user_account),
            "7" | "delete thread" => delete_thread(connection, &user_account),
            "8" | "delete account" => delete_account(connection, &user_account),
            "9" | "clear" => clear_screen(),
            "0" | "exit" => break,
            _ => continue
        }
        
    };
}
