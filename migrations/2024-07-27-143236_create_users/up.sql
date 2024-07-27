CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE users (
       id SERIAL PRIMARY KEY,
       username VARCHAR NOT NULL,
       hash VARCHAR NOT NULL,
       session_id UUID NOT NULL DEFAULT uuid_generate_v4()
)
