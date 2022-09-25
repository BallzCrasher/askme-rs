use std::env;
use std::io;
use std::io::Write;
use rpassword::read_password;

fn print_flush(s: &str) {
    print!("{s}");
    io::stdout().flush().expect("IO Error :: Unable to flush stdout");
}

fn clear_screen() {
    print_flush(&format!("{esc}[2J{esc}[1;1H", esc = 27 as char));
}

fn user_input<T: std::str::FromStr>() -> Result<T, <T as std::str::FromStr>::Err>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut inp = String::new();
    io::stdin().read_line(&mut inp).expect("IO error");
    return inp.trim().parse::<T>();
}


fn sign_in(database: &sqlite3::Connection) -> String { 

    //only letters and numbers to prevent sql injection
    fn check_name(name: &str) -> bool {
        for c in name.chars() { 
            if (c < 'a' || c > 'z') || (c < '0' || c > '9') ||  (c < 'A' || c > 'Z') { 
                return false;
            }
        }
        true
    }

    let account =
    loop {

        // get Username
        print!("Username: ");
        io::stdout().flush().expect("Flush");

        let user_name: String = user_input::<String>().unwrap().trim().to_owned();

        if check_name(&user_name) { 
            println!("User name should only contain letters and numbers");
            continue;
        }
        

        //get password
        print_flush("Password: ");

        let user_password = read_password().expect("IO Error :: Cannot Read Password");
        let user_password = sha256::digest(user_password.trim());
        let mut correct_password = false;
        
        //check if account exist in the database
        database.iterate(format!("SELECT password FROM accounts WHERE name == '{user_name}'"),
                         |password_arr| {
                             //println!("{password_arr:?}");
                             password_arr.get(0)
                                 .map(|node| node.1
                                             .map(|stored_password| {
                                                 if stored_password == user_password { 
                                                     correct_password = true;
                                                 }
                                             })
                                    );
                             true
                         }
        ).expect("Error");

        if correct_password { 
            break user_name;
        } else { 
            println!("Incorrect Username or Password");
        }
    };

    return account;
}

fn get_questions_vector(database: &sqlite3::Connection) -> Result<Vec< (u32,Option<String>, String) >, sqlite3::Error> { 
    let mut v: Vec< (u32,Option<String>, String) > = Vec::new();
    database.iterate("SELECT qst,by,id FROM questions",
                     |node| { 
                         let mut question_content = String::new();
                         let mut by_author = None;
                         let mut id: u32 = 0;

                         // get qst
                         node.get(0).map(|questions_arr|  
                            questions_arr.1
                                .map(|text| question_content = text.to_owned() )
                         );

                         //get by
                         node.get(1).map(|questions_arr| { 
                            by_author = questions_arr.1.map(|text| text.to_owned());
                         });

                         //get id
                         node.get(2).map(|questions_arr| { 
                             id = questions_arr.1.expect("Error No Id for question").trim().parse().expect("Id is not a number");
                         });

                         v.push((id,by_author, question_content));

                     true
                })?;
    Ok(v)
}

fn get_questions(database: &sqlite3::Connection) -> Result<(), sqlite3::Error>{ 
    let v = get_questions_vector(database)?;
    for (id, by_author, content) in v { 
        println!("({id}) {content} -- {}", by_author.unwrap_or("Anonymous".to_owned()));
    }
    Ok(())
}

fn check_exist_in_thread(database: &sqlite3::Connection, id: u32, thread_id: u32) -> bool {
    let mut exists = false;
    database.iterate(format!("SELECT id FROM answers WHERE id = {id} AND thread_id = {thread_id}")
                     , |node| {
                         if !node.is_empty() { exists = true; }
                         true
                     }).expect("Error Quering database for answers");
    exists
}

fn check_exist_thread(database: &sqlite3::Connection, id: u32) -> bool {
    let mut exists = false;
    database.iterate(format!("SELECT id FROM questions WHERE id = {id}")
                     , |node| {
                         if !node.is_empty() { exists = true; }
                         true
                     }).expect("Error Quering database for answers");
    exists
}

fn answer_thread(database: &sqlite3::Connection, thread_id: u32,to_id: u32,author: &str, ans: &str)  {
    database.execute(format!("INSERT INTO answers(to_id, thread_id, author, ans) 
                        VALUES(
                            {to_id},
                            {thread_id},
                            '{author}',
                            '{ans}'
                        );")).expect("Error inserting answer to database");
}

fn open_thread(database: &sqlite3::Connection, account: &String) {
    print_flush("Input the id of the thread you wish to open:");
    let thread_id  = user_input::<u32>();
    if thread_id.is_err() {
        println!("Invalid Input :: Enter a number");
        return;
    }
    let thread_id = thread_id.unwrap();
    if !check_exist_thread(database, thread_id) {
        println!("No such thread.");
        return;
    }

    fn open_answers(database: &sqlite3::Connection,thread_id :u32, answer_id: i32, layer: usize){ 
        database.iterate(format!("SELECT id,author,ans FROM answers 
                         WHERE thread_id = {thread_id} AND to_id = {answer_id}")
                         , |node| { 
                             let mut id = -1;
                             let mut author = None;
                             let mut content = String::new();
                             //get id
                             node.get(0).map(|questions_arr| { 
                                 id = questions_arr.1.expect("Error No Id for question").trim().parse().expect("Id is not a number");
                             });

                             //get author
                             node.get(1).map(|questions_arr| { 
                                author = questions_arr.1.map(|text| text.to_owned());
                             });

                             // get content
                             node.get(2).map(|questions_arr|  
                                questions_arr.1
                                    .map(|text| content = text.to_owned() )
                             );
                             print!("{}","\t".repeat(layer));
                             println!("({id}) {content} -- {}", author.unwrap_or("Anonymous".to_owned()));
                             if id != -1 { open_answers(database, thread_id, id, layer + 1); }

                            true
        }).expect("Error Getting Answers.");
    }

    open_answers(database, thread_id, 0, 0);

    loop {
        println!("Commands[ 0: back, 1: answer ]");
        print_flush("Command: ");
        let input = user_input::<String>().unwrap();

        match input.to_lowercase().trim() { 
            "1" | "answer" => {

                print_flush("input the id of which you wish to answer (0 if answering the original question):");
                let to_id = user_input::<u32>();
                if to_id.is_err() { 
                    println!("Invalid input :: Enter a number"); 
                    continue; 
                }
                let to_id = to_id.unwrap();

                if to_id == 0 || check_exist_in_thread(database, to_id, thread_id) {
                    let content = user_input::<String>().unwrap();
                    answer_thread(database, thread_id,to_id, account, content.trim());
                    open_answers(database, thread_id, 0, 0);
                } else {
                    println!("No such id exists");
                    continue;
                }

            },
            "0" | "back" => return,
            _ => continue
        }
    }
    
}

fn ask_question(database: &sqlite3::Connection, account: &String) -> Result<(), sqlite3::Error> { 
    let input = user_input::<String>().unwrap();
    let input = input.trim();
    database
        .execute(format!("INSERT INTO questions(qst, by) VALUES ('{input}', '{account}') ;"))?;

    Ok(())
}

fn init_database(database: &sqlite3::Connection) -> Result<(), sqlite3::Error> {
    database.execute("CREATE TABLE IF NOT EXISTS accounts(
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT,
                        password TEXT
                    );"
    )?;

    database.execute("CREATE TABLE IF NOT EXISTS questions(
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        qst TEXT,
                        by TEXT
                    );"
    )?;
    database.execute("CREATE TABLE IF NOT EXISTS answers(
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        to_id INTEGER,
                        thread_id, INTEGER,
                        author TEXT,
                        ans TEXT
                    );"
    )?;

    Ok(())
}

fn get_users(database: &sqlite3::Connection) -> Result<(), sqlite3::Error> {
    database.iterate("SELECT name FROM accounts", |node| {
             node.get(0).map(|name_tuple| {
                  name_tuple.1.map(|name| println!("{name}"))
             });
            true
    })?;

    Ok(())
}

fn main() -> Result<(), sqlite3::Error > {
    let database = sqlite3::open("database.db").expect("Couldn't connect to database.");
    init_database(&database)?;

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() == 3 { 
        let user_name = &args[1];
        let user_password = &args[2];
        let user_password = sha256::digest(user_password);
        database.execute(format!("INSERT INTO accounts(name, password) VALUES('{}', '{}')", user_name, user_password ))?;
    }

    let account = sign_in(&database);
    println!("Signed As: {account}");

    loop {
        println!("Commands: [ 1: questions, 2: ask, 3: open_thread, 4: users, 9: clear, 0: exit ]");
        print_flush("Command: "); 

        let input = user_input::<String>().unwrap();

        match input.to_lowercase().trim() { 
            "1" | "questions" => { get_questions(&database)?; },
            "2" | "ask" => { ask_question(&database, &account)?; },
            "3" | "open_thread" => { open_thread(&database, &account); },
            "4" | "users" => { get_users(&database)?; }
            "9" | "clear" => { clear_screen(); }
            "0" | "exit" => break,
            _ => continue
        }
        
    };

    Ok(())
}
