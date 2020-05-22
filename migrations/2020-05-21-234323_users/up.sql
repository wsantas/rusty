-- Your SQL goes here
create table users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email VARCHAR NOT NULL UNIQUE,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    access_token VARCHAR NOT NULL
)