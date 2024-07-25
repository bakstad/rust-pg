create table reports (
    id BIGSERIAL PRIMARY KEY,
    title TEXT,
    item_id BIGINT NOT NULL references items(id)
)