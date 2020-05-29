-- Your SQL goes here
create table users (
    id SERIAL,
    email VARCHAR(200),
    first_name VARCHAR(200),
    last_name VARCHAR(200),
    access_token VARCHAR(500)
)