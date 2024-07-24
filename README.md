# rust-pg

## Running locally

start docker

```
docker run -d -it -e POSTGRES_PASSWORD=password -p 5432:5432   postgres
```

run code:

```
cargo run --bin join_test
cargo run --bin show_posts
```