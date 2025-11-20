// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Integer,
        name -> Text,
        account_type -> Text,
        balance -> Float,
        user_id -> Integer,
    }
}

diesel::table! {
    contacts (id) {
        id -> Integer,
        name -> Text,
        user -> Integer,
    }
}

diesel::table! {
    transactions (id) {
        id -> Integer,
        user_account_id -> Integer,
        contact_id -> Integer,
        amount -> Float,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text,
        email -> Nullable<Text>,
    }
}

diesel::joinable!(accounts -> users (user_id));
diesel::joinable!(contacts -> users (user));
diesel::joinable!(transactions -> accounts (user_account_id));
diesel::joinable!(transactions -> contacts (contact_id));

diesel::allow_tables_to_appear_in_same_query!(accounts, contacts, transactions, users,);
