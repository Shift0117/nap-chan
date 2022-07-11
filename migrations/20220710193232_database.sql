-- Add migration script here
CREATE TABLE speakers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name NTEXT NOT NULL,
    style_id INTEGER NOT NULL,
    style_name NTEXT NOT NULL
)