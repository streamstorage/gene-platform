-- Your SQL goes here
CREATE TABLE users (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name varchar(32) NOT NULL,
  email varchar(32) NOT NULL,
  password varchar(64) NOT NULL,
  role smallint NOT NULL,
  active boolean NOT NULL,
  notes varchar(255) NULL,
  last_seen TIMESTAMP NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('users');

CREATE UNIQUE INDEX idx_email ON users (email);
