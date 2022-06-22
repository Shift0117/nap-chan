-- Add migration script here
CREATE TABLE dict (
    word TEXT NOT NULL PRIMARY KEY,
    read_word TEXT NOT NULL
)