create table reports (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    item_id INT NOT NULL references items(id)
)