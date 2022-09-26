use diesel::prelude::*;
use diesel::Insertable;

#[derive(Queryable)]
pub struct Account {
     pub id: i32,
     pub name: String,
     pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
     pub name: &'a str,
     pub password: &'a str,
}

#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub content: String,
    pub publisher: i32,
    pub is_anonymous: bool
}


#[derive(Insertable)]
#[diesel(table_name = questions)]
pub struct NewQuestion<'a> {
    pub content: &'a str,
    pub publisher: i32,
    pub is_anonymous: bool
}

#[derive(Queryable)]
pub struct Answer {
    pub id: i32,
    pub content: String,
    pub thread_id: i32,
    pub to_id: i32,
    pub publisher: i32,
    pub is_anonymous: bool
}

#[derive(Insertable)]
#[diesel(table_name = answers)]
pub struct NewAnswer<'a> {
    pub content: &'a str,
    pub thread_id: i32,
    pub to_id: i32,
    pub publisher: i32,
    pub is_anonymous: bool
}

diesel::table! {
    accounts (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
    }
}

diesel::table! {
    questions (id) {
        id -> Integer,
        content -> Text,
        publisher -> Integer,
        is_anonymous -> Bool,
    }
}

diesel::table! {
    answers (id) {
        id -> Integer,
        content -> Text,
        thread_id -> Integer,
        to_id -> Integer,
        publisher -> Integer,
        is_anonymous -> Bool,
    }
}
