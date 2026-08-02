// @generated automatically by Diesel CLI.

diesel::table! {
    series (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
