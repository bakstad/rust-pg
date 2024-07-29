create table invites (
    id BIGSERIAL PRIMARY KEY,
    kind VARCHAR NOT NULL,
    json jsonb not null
)