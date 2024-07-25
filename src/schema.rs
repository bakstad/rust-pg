// @generated automatically by Diesel CLI.

diesel::table! {
    address (id) {
        id -> Int4,
        value -> Text,
        author_id -> Int4,
    }
}

diesel::table! {
    authors (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
    }
}

diesel::table! {
    books_authors (book_id, author_id) {
        book_id -> Int4,
        author_id -> Int4,
    }
}

diesel::table! {
    items (id) {
        id -> Int4,
        title -> Text,
        num_plays -> Int4,
    }
}

diesel::table! {
    pages (id) {
        id -> Int4,
        page_number -> Int4,
        content -> Text,
        book_id -> Int4,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    reports (id) {
        id -> Int4,
        title -> Text,
        item_id -> Int4,
    }
}

diesel::joinable!(address -> authors (author_id));
diesel::joinable!(books_authors -> authors (author_id));
diesel::joinable!(books_authors -> books (book_id));
diesel::joinable!(pages -> books (book_id));
diesel::joinable!(reports -> items (item_id));

diesel::allow_tables_to_appear_in_same_query!(
    address,
    authors,
    books,
    books_authors,
    items,
    pages,
    posts,
    reports,
);
