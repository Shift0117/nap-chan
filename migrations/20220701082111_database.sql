-- Add migration script here
ALTER TABLE dict RENAME TO dict_tmp;
CREATE TABLE dict (
    word TEXT NOT NULL PRIMARY KEY,
    read_word TEXT NOT NULL
);
INSERT INTO dict(word,read_word) SELECT word,read_word FROM dict_tmp;
DROP TABLE dict_tmp;
ALTER TABLE user_config ADD COLUMN read_nickname TEXT