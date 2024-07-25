create table items (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    num_plays INTEGER NOT NULL default 0
)