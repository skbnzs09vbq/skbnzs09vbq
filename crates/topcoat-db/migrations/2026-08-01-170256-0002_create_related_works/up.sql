-- issue #7: 関連作品算出ロジックが参照する明示的リレーションテーブル。
-- `work_id` の Work に対して `related_work_id` を明示的な関連作品として登録する。
-- 明示的リレーションが1件も無い Work は、呼び出し側で共有タグ数によるフォールバック
-- 算出を行う。
CREATE TABLE related_works (
    work_id INTEGER NOT NULL REFERENCES works(id),
    related_work_id INTEGER NOT NULL REFERENCES works(id),
    PRIMARY KEY (work_id, related_work_id)
);
