// @generated automatically by Diesel CLI.
//
// NOTE: `works.id` は diesel print-schema の既定出力では `Nullable<Integer>` になるが
// （SQLite の `INTEGER PRIMARY KEY` は rowid エイリアスであり NULL 挿入時に自動採番される
// ため、diesel は保守的に nullable 扱いにする）、本プロジェクトでは `Work` 構造体1つに
// `Queryable`/`Selectable`/`Insertable` を共存させ、insert 時にも id を明示的に指定する
// 設計のため、手動で `Integer`（non-null）に補正している。
// `diesel migration run` で再生成した場合はこの1行のみ再度手動修正すること。

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
