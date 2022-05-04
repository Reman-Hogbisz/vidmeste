CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    user_id TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    displayname TEXT NOT NULL
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    video_id TEXT NOT NULL,
    video_path TEXT UNIQUE NOT NULL,
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


INSERT INTO users (email, user_id, displayname) VALUES ('hogbisz@gmail.com', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hogbisz');