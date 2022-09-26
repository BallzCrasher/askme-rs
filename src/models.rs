use diesel::prelude::*;
use diesel::Insertable;

use crate::schema::questions;
use crate::schema::accounts;
use crate::schema::answers;

#[derive(Queryable)]
pub struct Account {
     pub id: i32,
     pub name: String,
     pub password: String,
}

#[derive(Insertable)]
#[table_name = "accounts"]
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
#[table_name = "questions"]
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
    pub publisher: i32,
    pub is_anonymous: bool
}

#[derive(Insertable)]
#[table_name = "answers"]
pub struct NewAnswer<'a> {
    pub content: &'a str,
    pub thread_id: i32,
    pub publisher: i32,
    pub is_anonymous: bool
}
