use std::collections::HashMap;

use self::models::*;
use diesel::query_dsl::InternalJoinDsl;
use diesel::{debug_query, delete, pg::Pg, prelude::*, result::Error};
use rust_pg::{
    schema::{
        address, authors, books,
        books_authors::{self},
        pages,
    },
    *,
};

fn main() -> Result<(), Error> {
    let conn = &mut establish_connection();

    setup_data(conn)?;

    one_to_n_relations(conn)?;
    joins(conn)?;
    m_to_n_relations(conn)?;

    println!("-----------------");

    nested_join(conn)?;


    delete_all(conn)?;

    Ok(())
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct AddressAuthors<'a> {
    address: &'a Address,
    authors: Vec<AuthorBooks<'a>>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct AuthorBooks<'a> {
    author: &'a Author,
    books: Vec<&'a Book>,
}

fn nested_join(conn: &mut PgConnection) -> Result<(), Error> {
    let query = authors::table
        .inner_join(books_authors::table.inner_join(books::table))
        .inner_join(address::table)
        //        .filter(address::value.eq("sverige"))
        .select((Address::as_select(), Author::as_select(), Book::as_select()));

    println!("{}", debug_query::<Pg, _>(&query));

    let data = query.get_results::<(Address, Author, Book)>(conn)?;

    println!("XXXX: {data:?}");

    let mut m = HashMap::new();
    for (addr, auth, book) in data {
        m.entry(addr).or_insert_with(Vec::new).push((auth, book))
    }

    let mut mm = HashMap::new();
    for (addr, g) in m.iter() {
        let mut mmm = HashMap::new();
        for (auth, book) in g {
            mmm.entry(auth).or_insert_with(Vec::new).push(book)
        }

        mm.entry(addr).or_insert_with(Vec::new).push(mmm);
    }

    println!("____ {mm:?}");
    println!("____ {}", mm.len());

    let rs = mm
        .into_iter()
        .map(|(adr, rest)| AddressAuthors {
            address: adr,
            authors: rest
                .into_iter()
                .map(|xx| {
                    xx.into_iter()
                        .map(|(author, books)| AuthorBooks { author, books })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    println!(":::::: {:?}", rs);

    //    let x = data.iter().grouping_by(|(address, author, book)| address);
    //    println!("XXXX: {x:?}");

    Ok(())
}

fn delete_all(conn: &mut PgConnection) -> Result<(), Error> {
    diesel::delete(pages::table).execute(conn)?;
    diesel::delete(books_authors::table).execute(conn)?;
    diesel::delete(books::table).execute(conn)?;
    diesel::delete(address::table).execute(conn)?;
    diesel::delete(authors::table).execute(conn)?;

    let x = books::table
        .inner_join(pages::table)
        .select((Book::as_select(), Page::as_select()))
        .get_results::<(Book, Page)>(conn)?;

    Ok(())
}

fn joins(conn: &mut PgConnection) -> Result<(), Error> {
    let query = pages::table
        .inner_join(books::table)
        .filter(books::title.eq("Momo"))
        .select((Page::as_select(), Book::as_select()));

    println!("{}", debug_query::<Pg, _>(&query));

    let page_with_book = query.load::<(Page, Book)>(conn)?;

    println!("Page-Book pairs: {page_with_book:?}");

    let book_without_pages = books::table
        .left_join(pages::table)
        .select((Book::as_select(), Option::<Page>::as_select()))
        .load::<(Book, Option<Page>)>(conn)?;

    println!("Book-Page pairs (including empty books): {book_without_pages:?}");
    Ok(())
}

#[derive(Debug)]
struct BookWithPages {
    book: Book,
    pages: Vec<Page>,
}

fn one_to_n_relations(conn: &mut PgConnection) -> Result<(), Error> {
    let query = books::table
        .filter(books::title.eq("Momo"))
        .select(Book::as_select());
    println!("{}", debug_query::<Pg, _>(&query));

    let momo = query.get_result(conn)?;

    // get pages for the book "Momo"
    let query = Page::belonging_to(&momo).select(Page::as_select());
    println!("{}", debug_query::<Pg, _>(&query));

    let pages = query.load(conn)?;

    println!("Pages for \"Momo\": \n {pages:?}\n");

    let all_books = books::table.select(Book::as_select()).load(conn)?;

    // get all pages for all books
    let pages = Page::belonging_to(&all_books)
        .select(Page::as_select())
        .load(conn)?;

    // group the pages per book
    let pages_per_book = pages
        .grouped_by(&all_books)
        .into_iter()
        .zip(all_books)
        .map(|(pages, book)| (book, pages))
        .collect::<Vec<(Book, Vec<Page>)>>();

    println!("Pages per book: \n {pages_per_book:?}\n");

    let all_books = books::table.select(Book::as_select()).load(conn)?;

    // get all pages for all books
    let pages = Page::belonging_to(&all_books)
        .select(Page::as_select())
        .load(conn)?;

    let pages_per_book = pages
        .grouped_by(&all_books)
        .into_iter()
        .zip(all_books)
        .map(|(pages, book)| BookWithPages { book, pages })
        .collect::<Vec<BookWithPages>>();

    println!("Pages per book: \n {pages_per_book:?}\n");

    Ok(())
}

fn m_to_n_relations(conn: &mut PgConnection) -> Result<(), Error> {
    let astrid_lindgren = authors::table
        .filter(authors::name.eq("Astrid Lindgren"))
        .select(Author::as_select())
        .get_result(conn)?;

    // get all of Astrid Lindgren's books
    let query = BookAuthor::belonging_to(&astrid_lindgren)
        .inner_join(books::table)
        .select(Book::as_select());

    println!("{}", debug_query::<Pg, _>(&query));

    let books = query.load(conn)?;

    println!("Asgrid Lindgren books: {books:?}");

    let collaboration = books::table
        .filter(books::title.eq("Pippi and Momo"))
        .select(Book::as_select())
        .get_result(conn)?;

    // get authors for the collaboration
    let authors = BookAuthor::belonging_to(&collaboration)
        .inner_join(authors::table)
        .select(Author::as_select())
        .load(conn)?;
    println!("Authors for \"Pipi and Momo\": {authors:?}");

    // get a list of authors with all their books
    let all_authors = authors::table.select(Author::as_select()).load(conn)?;

    let books = BookAuthor::belonging_to(&authors)
        .inner_join(books::table)
        .select((BookAuthor::as_select(), Book::as_select()))
        .load(conn)?;

    let books_per_author: Vec<(Author, Vec<Book>)> = books
        .grouped_by(&all_authors)
        .into_iter()
        .zip(authors)
        .map(|(b, author)| (author, b.into_iter().map(|(_, book)| book).collect()))
        .collect();

    println!("All authors including their books: {books_per_author:?}");

    Ok(())
}

fn new_author(conn: &mut PgConnection, name: &str) -> Result<Author, Error> {
    let author = diesel::insert_into(authors::table)
        .values(authors::name.eq(name))
        .returning(Author::as_returning())
        .get_result(conn)?;
    Ok(author)
}

fn new_book(conn: &mut PgConnection, title: &str) -> Result<Book, Error> {
    let book = diesel::insert_into(books::table)
        .values(books::title.eq(title))
        .returning(Book::as_returning())
        .get_result(conn)?;
    Ok(book)
}

fn new_books_author(
    conn: &mut PgConnection,
    book_id: i32,
    author_id: i32,
) -> Result<BookAuthor, Error> {
    let book_author = diesel::insert_into(books_authors::table)
        .values((
            books_authors::book_id.eq(book_id),
            books_authors::author_id.eq(author_id),
        ))
        .returning(BookAuthor::as_returning())
        .get_result(conn)?;
    Ok(book_author)
}

fn new_page(
    conn: &mut PgConnection,
    page_number: i32,
    content: &str,
    book_id: i32,
) -> Result<Page, Error> {
    let page = diesel::insert_into(pages::table)
        .values((
            pages::page_number.eq(page_number),
            pages::content.eq(content),
            pages::book_id.eq(book_id),
        ))
        .returning(Page::as_returning())
        .get_result(conn)?;
    Ok(page)
}

fn new_address(conn: &mut PgConnection, address: &str, author_id: i32) -> Result<Address, Error> {
    let page = diesel::insert_into(address::table)
        .values((address::value.eq(address), address::author_id.eq(author_id)))
        .returning(Address::as_returning())
        .get_result(conn)?;
    Ok(page)
}

fn setup_data(conn: &mut PgConnection) -> Result<(), Error> {
    // create a book
    let momo = new_book(conn, "Momo")?;

    // a page in that book
    new_page(conn, 1, "In alten, alten Zeiten ...", momo.id)?;
    // a second page
    new_page(conn, 2, "den prachtvollen Theatern...", momo.id)?;

    // create an author
    let michael_ende = new_author(conn, "Michael Ende")?;

    // let's add the author to the already created book
    new_books_author(conn, momo.id, michael_ende.id)?;

    new_address(conn, "Derp road", michael_ende.id)?;

    // create a second author
    let astrid_lindgren = new_author(conn, "Astrid Lindgren")?;
    new_address(conn, "sverige", astrid_lindgren.id)?;

    let pippi = new_book(conn, "Pippi Långstrump")?;
    new_books_author(conn, pippi.id, astrid_lindgren.id)?;

    // now that both have a single book, let's add a third book, an imaginary collaboration
    let collaboration = new_book(conn, "Pippi and Momo")?;
    new_books_author(conn, collaboration.id, astrid_lindgren.id)?;
    new_books_author(conn, collaboration.id, michael_ende.id)?;

    Ok(())
}