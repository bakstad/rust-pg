CREATE TABLE address (
  id SERIAL PRIMARY KEY,
  value TEXT NOT NULL,
  author_id INTEGER NOT NULL REFERENCES authors(id)
);