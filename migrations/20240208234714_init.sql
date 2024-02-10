-- Add migration script here
CREATE TABLE users (userId STRING NOT NULL PRIMARY KEY, email STRING NOT NULL, username STRING NOT NULL, password STRING NOT NULL, genrePreferences STRING NOT NULL)
