CREATE TABLE user (
  id INT8 NOT NULL PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  pass_hash TEXT NOT NULL
);
CREATE TABLE session (
  token TEXT NOT NULL PRIMARY KEY,
  user_id INT8 NOT NULL,
  start INT8 NOT NULL,
  refresh INT8 NOT NULL
);
CREATE INDEX idx_user_id_of_session ON session(user_id);