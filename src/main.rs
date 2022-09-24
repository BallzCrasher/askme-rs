use std::env;
use std::io;
use std::io::Write;
use rpassword::read_password;

fn sign_up(database: &sqlite3::Connection) -> String { 

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

        let mut user_name = String::new();
        io::stdin()
            .read_line(&mut user_name)
            .expect("Error reading value");
        let user_name = user_name.trim().to_owned();

        if check_name(&user_name) { 
            println!("User name should only contain letters and numbers");
            continue;
        }
        

        //get password
        print!("Password: ");
        io::stdout().flush().expect("Flush");

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
    println!("Input the id of the thread you wish to open:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Read Error");
    let input = input.trim().parse::<u32>();
    if input.is_err() {
        println!("Invalid Input :: Enter a number");
        return;
    }
    let thread_id = input.unwrap();
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
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Read Error");

        match input.to_lowercase().trim() { 
            "1" | "answer" => {

                println!("input the id of which you wish to answer (0 if answering the original question):");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Read Error");
                let input = input.trim().parse::<u32>();
                if input.is_err() { println!("Invalid input :: Enter a number"); continue; }
                let to_id = input.unwrap();

                if to_id == 0 || check_exist_in_thread(database, to_id, thread_id) {
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Read Error");
                    answer_thread(database, thread_id,to_id, account, input.trim());
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
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Read Error");
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

    let account = sign_up(&database);
    println!("Signed As: {account}");

    loop {
        let mut input = String::new();
        println!("Commands: [ 1: questions, 2: ask, 3: open_thread, 4: get-users, 0: exit ]");
        print!("Command: "); io::stdout().flush().expect("Flush Error");
        io::stdin()
            .read_line(&mut input)
            .expect("Read Error");

        match input.to_lowercase().trim() { 
            "1" | "questions" => { get_questions(&database)?; },
            "2" | "ask" => { ask_question(&database, &account)?; },
            "3" | "open_thread" => { open_thread(&database, &account); },
            "0" | "exit" => break,
            _ => continue
        }
        
    };

    Ok(())
}
