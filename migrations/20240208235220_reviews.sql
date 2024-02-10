-- Add migration script here

CREATE TABLE reviews (reviewId STRING NOT NULL PRIMARY KEY, enjoyability INTEGER NOT NULL, educationalValue INTEGER NOT NULL, usability INTEGER NOT NULL, replayability INTEGER NOT NULL, content STRING, gameId INTEGER NOT NULL, userId STRING NOT NULL, FOREIGN KEY(userId) REFERENCES users(userId))
