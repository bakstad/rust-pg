use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;

pub trait Paginate: Sized {
    fn paginate(self, page: i64) -> Paginated<Self>;
}

pub trait PaginateWithTotal: Sized {
    fn paginate_with_total(self, page: i64) -> PaginatedWithTotal<Self>;
}

impl<T: Query> Paginate for T {
    fn paginate(self, page: i64) -> Paginated<Self> {
        let page = std::cmp::max(page, 1);

        Paginated {
            common: Common {
                query: self,
                per_page: DEFAULT_PER_PAGE,
                page,
                offset: (page - 1) * DEFAULT_PER_PAGE,
            },
        }
    }
}

impl<T: Query> PaginateWithTotal for T {
    fn paginate_with_total(self, page: i64) -> PaginatedWithTotal<Self> {
        let page = std::cmp::max(page, 1);

        PaginatedWithTotal {
            common: Common {
                query: self,
                per_page: DEFAULT_PER_PAGE,
                page,
                offset: (page - 1) * DEFAULT_PER_PAGE,
            },
        }
    }
}

const DEFAULT_PER_PAGE: i64 = 10;

#[derive(Debug, Clone, Copy, QueryId)]
struct Common<T> {
    query: T,
    page: i64,
    per_page: i64,
    offset: i64,
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    common: Common<T>,
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct PaginatedWithTotal<T> {
    common: Common<T>,
}

#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T> PaginatedWithTotal<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        let per_page = std::cmp::min(per_page, DEFAULT_PER_PAGE);

        PaginatedWithTotal {
            common: Common {
                per_page,
                offset: (self.common.page - 1) * per_page,
                ..self.common
            },
        }
    }

    // TODO: How to make this generic over all pairs, currently it only supports returning one datatype
    //   --> Seems to work with <tables>::all_columns, but not Table::as_select() for some reason
    pub fn load_and_count_pages<'a, U>(
        self,
        conn: &mut PgConnection,
    ) -> QueryResult<PaginatedResult<U>>
    where
        Self: LoadQuery<'a, PgConnection, (U, i64)>,
    {
        let per_page = self.common.per_page;
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.first().map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Ok(PaginatedResult {
            data: records,
            page_size: per_page,
            total_pages,
        })
    }
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated {
            common: Common {
                per_page,
                offset: (self.common.page - 1) * per_page,
                ..self.common
            },
        }
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = T::SqlType;
}

impl<T: Query> Query for PaginatedWithTotal<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}
impl<T> RunQueryDsl<PgConnection> for PaginatedWithTotal<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        self.common.query.walk_ast(out.reborrow())?;
        out.push_sql(" LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.common.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.common.offset)?;
        Ok(())
    }
}

impl<T> QueryFragment<Pg> for PaginatedWithTotal<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.common.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.common.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.common.offset)?;
        Ok(())
    }
}
