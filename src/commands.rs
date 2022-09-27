use diesel::{prelude::*, insert_into};
use crate::models::{Account, Question, Answer};
use crate::models::{NewQuestion, NewAnswer};
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
            use crate::models::accounts::dsl::*;
            let publisher_name = accounts
                .find(question.publisher)
                .first::<Account>(conn)
                .unwrap()
                .name;
                //
            println!("({}) {} -- {}", question.id, question.content,publisher_name);
        }
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

    get_questions(conn);
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
                    .first(conn).expect(format!("ERROR :: FOUND ID ({}) OF UNKNOWN USER",answer.publisher).as_str());

                println!("({}) {} -- {}", answer.id, answer.content,username);
            }
            show_thread(conn, thrd_id, answer.id, layer + 1)
        }
    }

    fn prompt_replay(conn: &mut SqliteConnection, thrd_id: i32, account_id: i32, anonymous: bool){ 
        use crate::models::answers::dsl::*;
        let input = user_input::<i32>("Enter the id of the post you wish to replay to (0 if replaying to the original post): ");
        if input.is_err() { println!("id must be a number"); return;}
        let input_id = input.unwrap();
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

    //get thread_id
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
    let question = result.unwrap(); 
    loop { 
        if question.is_anonymous { 
            println!("(0) {} -- {}", question.content, "Anonymous" )
        } else { 
            let publisher_account: Result<Account,_> = accounts
                .find(question.publisher)
                .first(conn);
            if publisher_account.is_err() { 
                println!("(0) {} -- {}", question.content, "Deleted" );
            } else { 
                println!("(0) {} -- {}", question.content, publisher_account.unwrap().name );
            }
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
