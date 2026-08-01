// @generated automatically by Diesel CLI.

diesel::table! {
    versions (id) {
        id -> Integer,
        work_id -> Integer,
        version_label -> Text,
        changelog -> Text,
        created_at -> Timestamp,
    }
}
