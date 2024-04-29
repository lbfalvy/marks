ALTER TABLE user ADD COLUMN layout TEXT NOT NULL DEFAULT '';
CREATE TABLE board (
  id INT8 NOT NULL PRIMARY KEY,
  url INT8 NOT NULL,
  name TEXT NOT NULL,
  version INT4 NOT NULL,
  owner_id INT8 NOT NULL,
  public_mut BOOL NOT NULL,
  layout TEXT NOT NULL
);
CREATE INDEX idx_url_of_board ON board(url);