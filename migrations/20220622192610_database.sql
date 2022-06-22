-- Add migration script here
CREATE TABLE user_config (
    user_id INT NOT NULL PRIMARY KEY,
    hello NTEXT NOT NULL DEFAULT "こんにちは",
    bye NTEXT NOT NULL DEFAULT "ばいばい",
    generator_type INT NOT NULL DEFAULT 0,
    voice_type INT NOT NULL DEFAULT 1
);

CREATE TABLE dict (
    word TEXT NOT NULL PRIMARY KEY,
    read_word TEXT NOT NULL
);