// @generated automatically by Diesel CLI.
//
// NOTE: `works.id` は diesel print-schema の既定出力では `Nullable<Integer>` になるが
// （SQLite の `INTEGER PRIMARY KEY` は rowid エイリアスであり NULL 挿入時に自動採番される
// ため、diesel は保守的に nullable 扱いにする）、本プロジェクトでは `Work` 構造体1つに
// `Queryable`/`Selectable`/`Insertable` を共存させ、insert 時にも id を明示的に指定する
// 設計のため、手動で `Integer`（non-null）に補正している。
// `diesel migration run` で再生成した場合はこの1行のみ再度手動修正すること。

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
    versions (id) {
        id -> Integer,
        work_id -> Integer,
        version_label -> Text,
        changelog -> Text,
        created_at -> Timestamp,
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
        series_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        thumbnail -> Nullable<Text>,
        params -> Nullable<Text>,
    }
}

diesel::joinable!(versions -> works (work_id));
diesel::joinable!(work_tags -> tags (tag_id));
diesel::joinable!(work_tags -> works (work_id));

diesel::allow_tables_to_appear_in_same_query!(
    related_works,
    tags,
    versions,
    work_tags,
    works,
);
