CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    video_id TEXT NOT NULL,
    video_path TEXT NOT NULL,
    video_name TEXT NOT NULL,
    video_length INTEGER NOT NULL,
    video_desc TEXT NOT NULL,
    owner_id SERIAL references users(id),
    thumbnail_path TEXT
);

CREATE TABLE one_time_video (
    id SERIAL PRIMARY KEY,
    video_id SERIAL references videos(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    one_time_pass TEXT NOT NULL
);


INSERT INTO users (email) VALUES ('hogbisz@gmail.com');