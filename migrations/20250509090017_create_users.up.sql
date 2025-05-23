CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    pid VARCHAR(36) UNIQUE NOT NULL,
    role SMALLINT CHECK (role BETWEEN 0 AND 7) NOT NULL,
    partition VARCHAR(200),
    partition_size BIGiNT,
    email VARCHAR(300),
    password VARCHAR(80),
    created_at VARCHAR(120) NOT NULL
);
