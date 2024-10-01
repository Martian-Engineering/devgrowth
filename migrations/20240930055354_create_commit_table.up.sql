-- Add migration script here
CREATE TABLE commit (
    commit_id SERIAL PRIMARY KEY,
    sha text NOT NULL,
    repository_id INTEGER NOT NULL,
    author text NOT NULL,
    date timestamptz NOT NULL,
    message text,
    FOREIGN KEY (repository_id) REFERENCES repository(repository_id),
    UNIQUE (repository_id, sha)
);
