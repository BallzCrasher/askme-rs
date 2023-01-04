use diesel::{prelude::*, insert_into};
use crate::models::{Account, Question, Answer};
use crate::models::{NewQuestion, NewAnswer};
use crate::user_input;

pub fn get_questions(conn: &mut SqliteConnection) {
    use crate::models::questions::dsl::*;
    let results: Vec<Question> = questions
        .load(conn)
        .expect("Error Quering for questions");

    let mut current_index = 1;
    for question in results {
        use crate::models::accounts::dsl::*;
        let publisher_account = accounts
            .find(question.publisher)
            .first::<Account>(conn);
        if publisher_account.is_err() { 
            println!("({}) {} -- {}", current_index, question.content, "Deleted");
        }
        if question.is_anonymous {
            println!("({}) {} -- {}", current_index, question.content, "Anonymous");
        } else {
            println!("({}) {} -- {}", current_index, question.content,publisher_account.unwrap().name);
        }
        current_index += 1;
    }
}

pub fn ask_question(conn: &mut SqliteConnection, account: &Account, anonymous: bool ) { 
    use crate::models::questions::dsl::*;
    let input = user_input::<String>("Question: ").unwrap();
    let qst = NewQuestion {
        content: input.as_str(),
        publisher: account.id,
        is_anonymous: anonymous
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
        println!("({}) -> {}", account.id, account.name);
    }
}

pub fn open_thread(conn: &mut SqliteConnection, account: &Account) {
    use crate::models::questions::dsl::*;
    use crate::models::accounts::dsl::*;

    fn show_thread(conn: &mut SqliteConnection, thrd_id: i32, replay_id: i32, layer: usize) { 
        use crate::models::answers::dsl::*;
        let results: Vec<Answer> = answers
            .filter(thread_id.eq(thrd_id))
            .filter(to_id.eq(replay_id))
            .load(conn).expect("Error connecting to database");

        for answer in results { 
            print!("{}", "\t".repeat(layer));
            if answer.is_anonymous { 
                println!("({}) {} -- {}", answer.id, answer.content, "Anonymous");
            } else { 
                let username: String = accounts
                    .select(name)
                    .find(answer.publisher)
                    .first(conn).unwrap_or(String::from("Deleted"));

                println!("({}) {} -- {}", answer.id, answer.content,username);
            }
            show_thread(conn, thrd_id, answer.id, layer + 1)
        }
    }

    fn prompt_replay(conn: &mut SqliteConnection, thrd_id: i32, account_id: i32, anonymous: bool){ 
        use crate::models::answers::dsl::*;
        let input_id = match user_input::<i32>("Enter the id of the post you wish to replay to (0 if replaying to the original post): ") {
            Ok(int) => int,
            Err(_) => { eprintln!("id must be a number"); return; }
        };
        // checking if there is post with such id
        if input_id != 0 && answers.find(input_id).first::<Answer>(conn).is_err() { 
            println!("No Such id");
            return;
        }
        // input is valid
        let user_content: String = user_input("replay: ").unwrap();

        insert_into(answers)
            .values(NewAnswer{ 
                content: &user_content,
                thread_id: thrd_id,
                to_id: input_id,
                publisher: account_id,
                is_anonymous: anonymous
            }).execute(conn).expect("Error connecting to database");
    }

    fn prompt_delete(conn: &mut SqliteConnection, thrd_id: i32, account_id: i32){ 
        use crate::models::answers::dsl::*;
        let input = user_input::<i32>("Enter the id of the post you wish to replay to (0 if replaying to the original post): ");
        if input.is_err() { println!("id must be a number"); return;}
        let input_id = input.unwrap();
        // checking if there is post with such id
        let result = answers
            .filter(thread_id.eq(thrd_id))
            .find(input_id)
            .first::<Answer>(conn);
        if  result.is_err() { 
            println!("No Such Answer id belonging to this thread");
            return;
        }
        // check if post belongs to the user
        let user_answer = result.unwrap();
        
        if user_answer.publisher == account_id { 
            diesel::delete(
                answers
                .filter(id.eq(user_answer.id))
            ).execute(conn).expect("Failed to delete the post :: connection error");
        } else { 
            println!("Cannot delete posts which do not belong to you.");
            return;
        }
    }

    get_questions(conn);
    //get thread_id
    let thread_id = match user_input::<i32>("Input the id of the thread: ") {
        Ok(int) => int,
        Err(_) => { eprintln!("Invalid Input::id must be a number"); return; }
    };
    let result = questions
        .offset((thread_id - 1) as i64)
        .first::<Question>(conn);
    // currently there is no way to handle sever errors
    let question = match result {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Invalid Input :: No thread with such id or invalid connection");
            return;
        }
    };

    loop { 
        let publisher_account: Result<Account,_> = accounts
            .find(question.publisher)
            .first(conn);
        if publisher_account.is_err() { 
            println!("(0) {} -- {}", question.content, "Deleted" );
        } else if question.is_anonymous { 
            println!("(0) {} -- {}", question.content, "Anonymous" )
        } else  { 
            println!("(0) {} -- {}", question.content, publisher_account.unwrap().name );
        }

        show_thread(conn, thread_id, 0, 1);
        println!("Commands: [ 0: back, 1: replay, 2: anonymous replay, 3: delete ]");
        let input: String = user_input("-~> ").unwrap();

        match input.to_lowercase().trim() { 
            "0"|"back" => break,
            "1"|"replay" => prompt_replay(conn,thread_id,account.id, false),
            "2"|"anonymous replay" => prompt_replay(conn,thread_id,account.id, true),
            "3"|"delete" => prompt_delete(conn, thread_id, account.id),
            _ => continue
        }
    }
}

pub fn user_answers(conn: &mut SqliteConnection, account: &Account){ 
    use crate::models::answers::dsl::*;
    let results: Vec<Answer> = answers
        .filter(publisher.eq(account.id))
        .load(conn)
        .expect("Error connecting to database");

    for answer in results { 
        println!("({}) {} -> Thread: {}",answer.id, answer.content, answer.thread_id);
    }
}

pub fn delete_thread(conn: &mut SqliteConnection, account: &Account){ 
    let thrd_id = user_input::<i32>("Enter the id of the thread you want to delete: ");
    if thrd_id.is_err() { println!("id must be a number."); return; }
    let thrd_id = thrd_id.unwrap();
    use crate::models::questions::dsl::*;
    // check if thread belongs to the user
    let result = questions
        .find(thrd_id)
        .first::<Question>(conn);
    if result.is_err() { 
        println!("No such ID");
        return;
    }
    let marked_thread = result.unwrap();
    if marked_thread.publisher != account.id { 
        println!("Can't delete a thread that does not belong to you");
        return;
    }
    // valid  
    use crate::models::answers::dsl;
    // delete all answers that belongs to this thread
    diesel::delete(dsl::answers.filter(dsl::thread_id.eq(thrd_id)))
        .execute(conn)
        .expect("connection error");
    // and finally delete the thread
    diesel::delete(questions.filter(id.eq(thrd_id)))
        .execute(conn)
        .expect("connection error");

}

pub fn delete_account(conn: &mut SqliteConnection, account: &Account) {
    let input: String = user_input("Do you want to delete all your questions and answers? [N/y]: ").unwrap();
    if  input.to_lowercase().trim().starts_with("y"){ 
        //delete answers
        use crate::models::answers::dsl::*;
        diesel::delete(
            answers
            .filter(publisher.eq(account.id))
        ).execute(conn).expect("Error connecting to database");
        // delete thread
        use crate::models::questions::dsl;
        let user_threads: Vec<i32> = dsl::questions
            .select(dsl::id)
            .filter(dsl::publisher.eq(account.id))
            .load(conn).expect("connection error");
        for thrd_id in user_threads { 
            // delete all answers that belongs to this thread
            diesel::delete(answers.filter(thread_id.eq(thrd_id)))
                .execute(conn)
                .expect("connection error");
        }
        // and finally delete the thread
        diesel::delete(dsl::questions.filter(dsl::publisher.eq(account.id)))
            .execute(conn)
            .expect("connection error");
    } else { return; }
    use crate::models::accounts::dsl::*;
        diesel::delete(accounts.filter(id.eq(account.id))
        ).execute(conn).expect("Error connecting to database");
    
    std::process::exit(0);
}
