use diesel::{AsExpression, FromSqlRow};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::diesel_jsonb;
use crate::schema::{address, authors, books_authors, items, posts, reports};
use crate::schema::{books, pages};

#[derive(Debug, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Hash, Eq, PartialEq, Clone)]
#[diesel(table_name = books)]
pub struct Book {
    pub id: i32,
    pub title: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Book))]
#[diesel(table_name = pages)]
pub struct Page {
    pub id: i32,
    pub page_number: i32,
    pub content: String,
    pub book_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Hash, Eq, PartialEq)]
#[diesel(table_name = authors)]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone)]
#[diesel(belongs_to(Book))]
#[diesel(belongs_to(Author))]
#[diesel(table_name = books_authors)]
#[diesel(primary_key(book_id, author_id))]
pub struct BookAuthor {
    pub book_id: i32,
    pub author_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Hash, Eq, PartialEq, Clone)]
#[diesel(belongs_to(Author))]
#[diesel(table_name = address)]
pub struct Address {
    pub id: i32,
    pub value: String,
    pub author_id: i32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Hash, Eq, PartialEq, Clone)]
#[diesel(table_name = items)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub num_plays: i32,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Item))]
#[diesel(table_name = reports)]
pub struct Report {
    pub id: i32,
    pub title: String,
    pub item_id: i32,
}

#[derive(Debug, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invite {
    pub id: i64,
    pub kind: String,
    pub json: serde_json::Value,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::invites)]
pub struct InviteJson {
    pub id: i64,
    pub kind: String,
    pub json: InviteData,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::invites)]
pub struct NewInviteJson {
    pub kind: String,
    pub json: InviteData,
}

use diesel::pg::sql_types::Jsonb;

#[derive(Debug, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Jsonb)]
#[serde(tag = "kind")]
pub enum InviteData {
    Email { name: String },
    Link { url: String },
}
diesel_jsonb!(InviteData);
