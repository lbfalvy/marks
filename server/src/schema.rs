// @generated automatically by Diesel CLI.

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
    }
}

diesel::allow_tables_to_appear_in_same_query!(session, user,);
