table! {
    tasks (id) {
        id -> Nullable<Integer>,
        description -> Text,
        completed -> Bool,
    }
}

table! {
    users (id) {
        id -> Nullable<Integer>,
        email -> Text,
        first_name -> Text,
        last_name -> Text,
        access_token -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    tasks,
    users,
);
