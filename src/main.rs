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

fn get_questions(database: &sqlite3::Connection) -> Result<Vec<String>, sqlite3::Error> { 
    let mut v: Vec<String> = Vec::new();
    database.iterate("SELECT qst FROM questions",
                     |node| { 
                        node.get(0).map(|questions_arr| { 
                            questions_arr.1.map(|text| v.push(text.to_owned()))
                        });
                        true
                }
            )?;

    Ok(v)
}

fn ask_question(database: &sqlite3::Connection) -> Result<(), sqlite3::Error> { 
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Read Error");
    database
        .execute(format!("INSERT INTO questions(qst) VALUES ('{input}') ;"))?;
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
                        qst TEXT
                    );"
    )?;
    database.execute("CREATE TABLE IF NOT EXISTS answers(
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        qst_id INTEGER,
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
        println!("Commands: [ 1: questions, 2: ask, 3: answer, 4: get-connected, 0: exit ]");
        print!("Command: "); io::stdout().flush().expect("Flush Error");
        io::stdin()
            .read_line(&mut input)
            .expect("Read Error");

        match input.to_lowercase().trim() { 
            "1" | "questions" => { println!("{:?}",get_questions(&database)?); },
            "2" | "ask" => { ask_question(&database)?; },
            "0" | "exit" => break,
            _ => continue
        }
        
    };

    Ok(())
}
