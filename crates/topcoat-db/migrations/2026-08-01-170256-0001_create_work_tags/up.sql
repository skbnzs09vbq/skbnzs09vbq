-- `work_tags` 中間テーブル: `works` と `tags` の多対多リレーションを表す。
-- issue #7（関連作品算出ロジック）の実装・検証に必要な範囲に限定した暫定版として
-- issue #2 未マージの間に先行作成されたが、issue #2（Tagエンティティと Work-Tag
-- 多対多リレーション実装）でカラム構成の変更なく正式実装として引き継がれた
-- （Diesel モデル・クエリ関数は `crate::models::tag`・`crate::queries::tag` を参照）。
CREATE TABLE work_tags (
    work_id INTEGER NOT NULL REFERENCES works(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (work_id, tag_id)
);
