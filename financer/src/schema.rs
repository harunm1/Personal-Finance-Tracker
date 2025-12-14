// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Integer,
        name -> Text,
        account_type -> Text,
        balance -> Float,
        user_id -> Integer,
        active -> Bool,
    }
}

diesel::table! {
    recurring_transactions (id) {
        id -> Integer,
        user_id -> Integer,
        account_id -> Integer,
        contact_id -> Integer,
        amount -> Float,
        category -> Text,
        next_run_at -> Text,
        frequency -> Text,
        active -> Bool,
    }
}

diesel::table! {
    recurring_transfers (id) {
        id -> Integer,
        user_id -> Integer,
        from_account_id -> Integer,
        to_account_id -> Integer,
        amount -> Float,
        next_run_at -> Text,
        frequency -> Text,
        active -> Bool,
    }
}

diesel::table! {
    budgets (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        category -> Text,
        limit_cents -> Integer,
        period -> Text,
        target_type -> Text,
        active -> Bool,
        updated_at -> Timestamp,
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
        category -> Text,
        date -> Text,
        amount_cents -> Integer,
        balance_after -> Float,
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
diesel::joinable!(budgets -> users (user_id));
diesel::joinable!(contacts -> users (user));
diesel::joinable!(recurring_transactions -> users (user_id));
diesel::joinable!(recurring_transactions -> accounts (account_id));
diesel::joinable!(recurring_transfers -> users (user_id));
diesel::joinable!(transactions -> accounts (user_account_id));
diesel::joinable!(transactions -> contacts (contact_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    budgets,
    contacts,
    recurring_transactions,
    recurring_transfers,
    transactions,
    users,
);
