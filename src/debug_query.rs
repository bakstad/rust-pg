use diesel::debug_query;
use diesel::pg::Pg;
use diesel::query_builder::QueryFragment;

pub trait DebugQuery {
    fn debug_query(self) -> Self;
}

impl<T: QueryFragment<Pg>> DebugQuery for T {
    fn debug_query(self) -> Self {
        println!("{}", debug_query::<Pg, _>(&self));

        self
    }
}
