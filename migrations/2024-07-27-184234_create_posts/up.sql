CREATE TYPE post_type AS ENUM ('url', 'text');
CREATE TABLE posts (
       id SERIAL PRIMARY KEY,
       user_id INTEGER NOT NULL REFERENCES users(id),
       title VARCHAR NOT NULL,
       post_type post_type NOT NULL,
       content VARCHAR NOT NULL
)
