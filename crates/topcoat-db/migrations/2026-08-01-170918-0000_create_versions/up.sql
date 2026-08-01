-- `versions` テーブル: 1つの Work（`works.id`）に対して複数の Version を1対多で保持する。
-- 作品の変遷履歴・changelog を表す。`works` テーブル自体は別 issue（#6）で追加されるが、
-- SQLite は CREATE TABLE 時に FOREIGN KEY 参照先テーブルの存在を検証しないため、
-- 先行してこのテーブルを定義しても問題ない。
CREATE TABLE versions (
  id INTEGER NOT NULL PRIMARY KEY,
  work_id INTEGER NOT NULL,
  version_label TEXT NOT NULL,
  changelog TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (work_id) REFERENCES works (id)
);

-- work_id 指定で時系列順（created_at）に取得するクエリを高速化するためのインデックス。
CREATE INDEX idx_versions_work_id_created_at ON versions (work_id, created_at);
