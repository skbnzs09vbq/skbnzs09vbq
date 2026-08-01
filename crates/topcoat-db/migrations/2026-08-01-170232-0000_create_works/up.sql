-- 暫定実装: 本来この `works` テーブルは issue #6（Workエンティティのテーブル定義と
-- Dieselモデル実装）で追加される想定だが、2026-08-02 時点で #6 が未マージのため、
-- issue #7（関連作品算出ロジック）の実装・検証に必要な範囲に限定した暫定版として
-- 本マイグレーションで先行作成する。
--
-- #6 マージ時は、series_id・params 等 #6 で定義される残りのカラムを追加マイグレーションで
-- 揃えること（このテーブル自体を作り直す必要はない想定）。
CREATE TABLE works (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    thumbnail TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
