-- `series_id` に本来つけたい `REFERENCES series(id)` は、`series` テーブルが
-- 未実装（issue #1、未着手）の間は付与しない。SQLite は CREATE TABLE 時点では
-- FK 参照先テーブルの実在を検証しないが、Diesel の SqliteConnection は接続確立時に
-- `PRAGMA foreign_keys = ON` を有効化するため、FK 宣言があると（series_id が NULL の
-- 行であっても）以降の works への INSERT/UPDATE/DELETE すべてが
-- "no such table: main.series" で失敗してしまう（実際に発生を確認済み）。
-- series テーブル追加（issue #1）のマイグレーションで、works テーブルを
-- 作り直すかたちで REFERENCES を付与し直すこと。
CREATE TABLE works (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    series_id INTEGER,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    thumbnail TEXT,
    params TEXT
);

