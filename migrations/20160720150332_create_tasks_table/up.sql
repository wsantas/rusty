CREATE TABLE tasks (
    id SERIAL,
    description VARCHAR(200),
    completed BOOLEAN
);

INSERT INTO tasks (description) VALUES ("demo task");
INSERT INTO tasks (description) VALUES ("demo task2");
