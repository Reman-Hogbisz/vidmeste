
CREATE TABLE user_permissions (
  id SERIAL PRIMARY KEY,
  permission VARCHAR(255) NOT NULL
);

INSERT INTO user_permissions (permission) VALUES ('admin');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    user_id TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    displayname TEXT NOT NULL,
    permissions INTEGER[] NOT NULL DEFAULT '{}'
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    video_id TEXT NOT NULL,
    video_path TEXT UNIQUE NOT NULL,
    video_url TEXT UNIQUE NOT NULL,
    video_name TEXT NOT NULL,
    video_length FLOAT NOT NULL,
    video_desc TEXT NOT NULL,
    owner_id SERIAL references users(id),
    thumbnail_path TEXT
);

CREATE TABLE video_shares (
    id SERIAL PRIMARY KEY,
    video_id SERIAL references videos(id),
    user_id SERIAL references users(id)
);

CREATE TABLE one_time_video (
    id SERIAL PRIMARY KEY,
    video_id SERIAL references videos(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    one_time_pass TEXT NOT NULL
);


INSERT INTO users (email, user_id, displayname, permissions) VALUES ('hogbisz@gmail.com', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hogbisz', '{1}');
INSERT INTO users (email, user_id, displayname, permissions) VALUES ('sjrembisz07@gmail.com', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab', 'Samuel Rembisz', '{1}');
INSERT INTO users (email, user_id, displayname, permissions) VALUES ('tehcakecore@gmail.com', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac', 'Samuel Rembisz', '{}');