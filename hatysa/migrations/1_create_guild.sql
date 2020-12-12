CREATE TABLE IF NOT EXISTS guild (
    -- This is a TEXT because guild IDs are 64-bit snowflake values, and sqlite only stores unsigned 
    -- integers.
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);
