-- 暫定実装: 本来この `tags` テーブルは issue #2（Tagエンティティと Work-Tag 多対多
-- リレーション実装）で追加される想定だが、2026-08-02 時点で #2 が未マージのため、
-- issue #7（関連作品算出ロジック）の実装・検証に必要な範囲に限定した暫定版として
-- 本マイグレーションで先行作成する。
CREATE TABLE tags (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE
);
