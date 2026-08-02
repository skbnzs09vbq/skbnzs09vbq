-- 暫定実装: 本来この `work_tags` 中間テーブルは issue #2（Tagエンティティと Work-Tag
-- 多対多リレーション実装）で追加される想定だが、2026-08-02 時点で #2 が未マージのため、
-- issue #7（関連作品算出ロジック）の実装・検証に必要な範囲に限定した暫定版として
-- 本マイグレーションで先行作成する。
CREATE TABLE work_tags (
    work_id INTEGER NOT NULL REFERENCES works(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (work_id, tag_id)
);
