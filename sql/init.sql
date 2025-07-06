CREATE DATABASE anarcy_core;

CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       login VARCHAR(50) UNIQUE NOT NULL,
                       hashed_password VARCHAR(255) NOT NULL
);

CREATE TABLE players (
                         user_id INTEGER PRIMARY KEY REFERENCES users(id),
                         x FLOAT NOT NULL,
                         y FLOAT NOT NULL,
                         z FLOAT NOT NULL DEFAULT 0
);