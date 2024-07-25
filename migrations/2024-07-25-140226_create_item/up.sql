create table items (
    id BIGSERIAL PRIMARY KEY,
    title TEXT,
    num_plays INTEGER default 0
)