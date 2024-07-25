use std::collections::HashMap;

use diesel::expression::{AsExpression, SqlLiteral, UncheckedBind};
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::{Bool, Integer};
use diesel::{debug_query, pg::Pg, prelude::*, result::Error};
use rand::Rng;

use self::models::*;
use rust_pg::debug_query::DebugQuery;
use rust_pg::pagination::{Paginate, PaginateWithTotal};
use rust_pg::schema::reports;
use rust_pg::{
    schema::{
        address, authors, books,
        books_authors::{self},
        items, pages,
    },
    *,
};

fn main() -> Result<(), Error> {
    let conn = &mut establish_connection();

    setup_data(conn)?;

    setup_items(conn)?;

    one_to_n_relations(conn)?;
    joins(conn)?;
    m_to_n_relations(conn)?;

    println!("-----------------");

    nested_join(conn)?;

    println!("-----------------");
    play_with_joins(conn)?;
    println!("-----------------");
    pagination_testing(conn)?;
    println!("-----------------");

    // reports_testing(conn)?;
    println!("-----------------");

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

fn pagination_testing(conn: &mut PgConnection) -> Result<(), Error> {
    // Paginate and return total
    let mut page = 1;

    loop {
        let query = books::table
            .select(Book::as_select())
            .paginate_with_total(page)
            .per_page(3);

        page += 1;

        // println!("{}", debug_query::<Pg, _>(&query));

        let books_pagination = query.load_and_count_pages(conn)?;

        if books_pagination.data.is_empty() {
            break;
        }

        println!("books_pagination_with_total: {:?}", books_pagination);
    }

    println!("###################################");
    println!("###################################");

    // Paginate without counting total
    let mut page = 1;

    loop {
        let query = books::table
            .select(Book::as_select())
            .paginate(page)
            .per_page(3);

        page += 1;

        // println!("{}", debug_query::<Pg, _>(&query));

        let books_pagination = query.load(conn)?;

        if books_pagination.is_empty() {
            break;
        }

        println!("books_pagination: {:?}", books_pagination);
    }

    println!("###################################");
    println!("###################################");

    // Paginate with join and return total
    let mut page = 1;

    loop {
        let query = books::table
            .inner_join(pages::table)
            .select((Book::as_select(), Page::as_select()))
            .paginate(page)
            .per_page(3);

        page += 1;

        // println!("{}", debug_query::<Pg, _>(&query));

        let books_pagination = query.load::<(Book, Page)>(conn)?;

        if books_pagination.is_empty() {
            break;
        }

        println!("books_pages_pagination: {:?}", books_pagination);
    }

    println!("###################################");
    println!("###################################");

    let count = books::table.count().get_result::<i64>(conn)?;
    println!("count: {:?}", count);

    println!("###################################");
    println!("###################################");

    // Paginate with join and return total
    let mut page = 1;

    loop {
        let result = books::table
            .inner_join(pages::table)
            .order_by((
                pages::page_number.desc(),
                books::id.desc(),
                pages::id.desc(),
            ))
            .select((books::all_columns, pages::all_columns))
            .paginate_with_total(page)
            .per_page(3)
            // .debug_query()
            .load_and_count_pages::<(Book, Page)>(conn)?;

        if result.data.is_empty() {
            break;
        }

        println!("Page {}/{}", page, result.total_pages);
        for (book, page) in result.data {
            println!(
                "Book({}) - Page({}) - page_nr: {}",
                book.id, page.id, page.page_number
            );
        }

        page += 1;
    }

    println!("###################################");
    println!("# Cursor based");
    println!("###################################");

    // Cursor pagination with join/multiple sort params

    let mut cursor = Cursor {
        book_id: 0,
        page_id: 0,
        page_number: 0,
        first: true,
    };

    loop {
        let cursor_filter =
            diesel::dsl::sql::<Bool>("((pages.page_number, books.id, pages.id) < (")
                .bind::<Integer, _>(cursor.page_number)
                .sql(", ")
                .bind::<Integer, _>(cursor.book_id)
                .sql(", ")
                .bind::<Integer, _>(cursor.page_id)
                .sql(") or ")
                .bind::<Bool, _>(cursor.first)
                .sql(")");
        // create_cursor(&mut cursor);

        let result = books::table
            .inner_join(pages::table)
            .filter(cursor_filter)
            .order_by((
                pages::page_number.desc(),
                books::id.desc(),
                pages::id.desc(),
            ))
            .select((books::all_columns, pages::all_columns))
            .limit(3)
            // .debug_query()
            .load::<(Book, Page)>(conn)?;

        if result.is_empty() {
            break;
        }

        let mut next_cursor = Cursor::default();
        if let Some((last_book, last_page)) = result.last() {
            next_cursor.book_id = last_book.id;
            next_cursor.page_id = last_page.id;
            next_cursor.page_number = last_page.page_number;
            next_cursor.first = false;
        }

        println!("cursor:      {:?}", cursor);
        println!("next_cursor: {:?}", next_cursor);
        for (book, page) in result {
            println!(
                "Book({}) - Page({}) - page_nr: {}",
                book.id, page.id, page.page_number
            );
        }

        cursor = next_cursor;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct Cursor {
    book_id: i32,
    page_id: i32,
    page_number: i32,
    first: bool,
}

fn reports_testing(conn: &mut PgConnection) -> Result<(), Error> {
    println!("##########################");
    println!("# REPORTS");
    println!("##########################");

    let mut page = 1;
    loop {
        let reports = reports::table
            .inner_join(items::table)
            .order_by((items::num_plays.desc(), reports::id.desc()))
            .select((Report::as_select(), Item::as_select()))
            .paginate(page)
            .load::<(Report, Item)>(conn)?;

        if reports.is_empty() {
            break;
        }

        println!("Page {}:", page);

        for (report, item) in reports {
            println!(
                "report {}, item: {}, plays: {}",
                report.id, item.id, item.num_plays
            );
        }
        println!();

        page += 1;
    }

    Ok(())
}

fn play_with_joins(conn: &mut PgConnection) -> Result<(), Error> {
    let books_pages = books::table
        .inner_join(pages::table)
        .filter(pages::content.similar_to("den%"))
        .order(pages::id.asc())
        .select((Book::as_select(), Page::as_select()))
        .load::<(Book, Page)>(conn)?;

    println!("books_pages: {:?}", books_pages);

    // select * from books b
    // join pages p on b.id = p.book_id
    // join books_authors ba on ba.book_id = b.id
    // join authors a on a.id = ba.author_id;

    let books_pages_authors = books::table
        .inner_join(pages::table)
        .inner_join(books_authors::table.inner_join(authors::table))
        .select((Book::as_select(), Page::as_select(), Author::as_select()))
        .load::<(Book, Page, Author)>(conn)?;

    println!("books_pages_authors: {:?}", books_pages_authors);

    Ok(())
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

    diesel::delete(reports::table).execute(conn)?;
    diesel::delete(items::table).execute(conn)?;

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

    new_page(conn, 1, "pipp1", pippi.id)?;
    new_page(conn, 2, "pipp1", pippi.id)?;

    // now that both have a single book, let's add a third book, an imaginary collaboration
    let collaboration = new_book(conn, "Pippi and Momo")?;
    new_books_author(conn, collaboration.id, astrid_lindgren.id)?;
    new_books_author(conn, collaboration.id, michael_ende.id)?;

    new_page(conn, 1, "momopipp1", collaboration.id)?;
    new_page(conn, 2, "momopipp2", collaboration.id)?;

    for i in 1..20 {
        new_book(conn, &format!("Book {}", i))?;
    }

    Ok(())
}

fn new_item(conn: &mut PgConnection, title: &str, num_plays: i32) -> Result<Item, Error> {
    let item = diesel::insert_into(items::table)
        .values((items::title.eq(title), items::num_plays.eq(num_plays)))
        .returning(Item::as_returning())
        .get_result(conn)?;

    Ok(item)
}

fn new_report(conn: &mut PgConnection, title: &str, item_id: i32) -> Result<Report, Error> {
    let item = diesel::insert_into(reports::table)
        .values((reports::title.eq(title), reports::item_id.eq(item_id)))
        .returning(Report::as_returning())
        .get_result(conn)?;

    Ok(item)
}

fn setup_items(conn: &mut PgConnection) -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    for i in 1..100 {
        let num_plays = rng.gen_range(0..1000);

        let item = new_item(conn, &format!("item {}", i), num_plays)?;

        for r in 1..3 {
            new_report(conn, &format!("report {} - {}", r, item.title), item.id)?;
        }
    }

    Ok(())
}
