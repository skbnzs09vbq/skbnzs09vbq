-- NOTE (issue #6 / #7 マージ時の統合): この works テーブルは issue #6 が正式に定義する
-- 完全なスキーマ（series_id・params・thumbnail 等を含む）。issue #7 が先行作成していた
-- 暫定版 works テーブル（旧 `2026-08-01-170232-0000_create_works`）はこのマイグレーションに
-- 統合・置き換えられたため削除した。tags/work_tags/related_works（issue #7 側）が
-- `REFERENCES works(id)` を持つため、このマイグレーションはそれらより前に実行される
-- タイムスタンプを付けている。
--
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

