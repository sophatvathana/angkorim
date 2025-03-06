-- Add delivery and read status fields to messages table
ALTER TABLE messages ADD COLUMN delivered BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE messages ADD COLUMN read BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE messages ADD COLUMN delivered_at INTEGER;
ALTER TABLE messages ADD COLUMN read_at INTEGER;

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_messages_delivered ON messages(delivered);
CREATE INDEX IF NOT EXISTS idx_messages_read ON messages(read);
CREATE INDEX IF NOT EXISTS idx_messages_delivered_at ON messages(delivered_at);
CREATE INDEX IF NOT EXISTS idx_messages_read_at ON messages(read_at);
