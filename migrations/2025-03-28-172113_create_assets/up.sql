-- Your SQL goes here
CREATE TABLE assets(
    id SERIAL PRIMARY KEY,
    asset_path VARCHAR NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public BOOLEAN DEFAULT FALSE NOT NULL
)