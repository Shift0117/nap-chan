-- Add migration script here
ALTER TABLE speakers RENAME TO speakers_tmp;
CREATE TABLE speakers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name NTEXT NOT NULL,
    style_id INTEGER NOT NULL,
    style_name NTEXT NOT NULL,
    generator_type INT NOT NULL
)