CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  sender TEXT NOT NULL,
  receiver TEXT NOT NULL,
  content TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  message_type INTEGER NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  origin_node TEXT NOT NULL DEFAULT 'user'
);
