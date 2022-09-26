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
        publisher -> Integer,
        is_anonymous -> Bool,
    }
}
