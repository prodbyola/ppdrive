-- Your SQL goes here
CREATE TABLE user_permissions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission SMALLINT CHECK (permission BETWEEN 0 AND 255) NOT NULL
)