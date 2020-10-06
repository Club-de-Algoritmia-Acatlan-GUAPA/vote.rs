DROP TABLE IF EXISTS users cascade;
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE
);

CREATE UNIQUE INDEX uname ON users(username);

DROP TABLE IF EXISTS items cascade;
CREATE TABLE items (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    done BOOL NOT NULL DEFAULT false
);

DROP TABLE IF EXISTS votes cascade;
CREATE TABLE votes (
    user_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    ordinal INTEGER NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(item_id) REFERENCES items(id)
);
CREATE UNIQUE INDEX no_dup_votes ON votes(user_id, item_id);
CREATE INDEX ballot ON votes(user_id ASC, ordinal ASC);

INSERT INTO items (title, body) VALUES ('Lunes', '13:00-14:00, Instructores disponibles: 3');
INSERT INTO items (title, body) VALUES ('Lunes', '14:00-15:00, Instructores disponibles: 1');

INSERT INTO items (title, body) VALUES ('Martes', '13:00-14:00, Instructores disponibles: 3');
INSERT INTO items (title, body) VALUES ('Martes', '19:00-20:00, Instructores disponibles: 3');

INSERT INTO items (title, body) VALUES ('Miércoles', '13:00-14:00, Instructores disponibles: 3');
INSERT INTO items (title, body) VALUES ('Miércoles', '14:00-15:00, Instructores disponibles: 1');

INSERT INTO items (title, body) VALUES ('Jueves', '13:00-14:00, Instructores disponibles: 3');

INSERT INTO items (title, body) VALUES ('Viernes', '13:00-14:00, Instructores disponibles: 3');
INSERT INTO items (title, body) VALUES ('Viernes', '14:00-15:00, Instructores disponibles: 3');
INSERT INTO items (title, body) VALUES ('Viernes', '15:00-16:00, Instructores disponibles: 3');
