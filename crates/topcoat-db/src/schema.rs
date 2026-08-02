// @generated automatically by Diesel CLI.

diesel::table! {
    related_works (work_id, related_work_id) {
        work_id -> Integer,
        related_work_id -> Integer,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
    }
}

diesel::table! {
    work_tags (work_id, tag_id) {
        work_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    works (id) {
        id -> Integer,
        title -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        thumbnail -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(work_tags -> tags (tag_id));
diesel::joinable!(work_tags -> works (work_id));

diesel::allow_tables_to_appear_in_same_query!(related_works, tags, work_tags, works,);
