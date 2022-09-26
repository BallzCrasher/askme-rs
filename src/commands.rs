use diesel::{prelude::*, insert_into};
use crate::models::*;
use crate::user_input;

pub fn get_questions(conn: &mut SqliteConnection) {
    use crate::models::questions::dsl::*;
    let results: Vec<Question> = questions
        .load(conn)
        .expect("Error Quering for questions");

    for question in results {
        if question.is_anonymous {
            println!("({}) {} -- {}", question.id, question.content, "Anonymous");
        } else {
            println!("({}) {} -- {}", question.id, question.content, question.publisher);
        }
    }
}

pub fn ask_question(conn: &mut SqliteConnection, account: &Account) { 
    use crate::models::questions::dsl::*;
    let input = user_input::<String>("Question: ").unwrap();
    let qst = NewQuestion {
        content: input.as_str(),
        publisher: account.id,
        is_anonymous: false
    };

    insert_into(questions)
        .values(&qst)
        .execute(conn)
        .expect("Error inserting to database.");
}

pub fn get_users(conn: &mut SqliteConnection) {
    use crate::models::accounts::dsl::*;
    let results: Vec<Account> = accounts
        .load(conn)
        .expect("Error getting accounts");

    for account in results {
        println!("{}", account.name);
    }
}

pub fn open_thread(conn: &mut SqliteConnection) {
    use crate::models::questions::dsl::*;

    let thread_id = user_input::<i32>("Input the id of the thread: ");
    if thread_id.is_err() {
        println!("Invalid Input::id must be a number");
        return;
    }
    let thread_id = thread_id.unwrap();

    let result = questions
        .find(thread_id)
        .first::<Question>(conn);
    // currently there is no way to handle sever errors
    if result.is_err() {
        println!("Invalid Input::No thread with such id");
        return;
    }
    
    println!("Commands[ 0: back, 1: post ]");
}
