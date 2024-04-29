// @generated automatically by Diesel CLI.

diesel::table! {
    board (id) {
        id -> BigInt,
        url -> BigInt,
        name -> Text,
        version -> Integer,
        owner_id -> BigInt,
        public_mut -> Bool,
        layout -> Text,
    }
}

diesel::table! {
    session (token) {
        token -> Text,
        user_id -> BigInt,
        start -> BigInt,
        refresh -> BigInt,
    }
}

diesel::table! {
    user (id) {
        id -> BigInt,
        name -> Text,
        pass_hash -> Text,
        layout -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(board, session, user,);
