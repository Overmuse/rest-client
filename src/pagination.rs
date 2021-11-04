use crate::Request;

#[derive(Debug, Clone)]
pub enum PaginationType {
    Query(Vec<(String, String)>),
}

pub trait Paginator<T> {
    fn next(
        &self,
        prev: &PaginationState<PaginationType>,
        res: &T,
    ) -> PaginationState<PaginationType>;
}

pub trait PaginatedRequest: Request {
    fn paginator(&self) -> Box<dyn Paginator<Self::Response>>;
}

#[derive(Clone, Debug)]
pub enum PaginationState<T: Clone> {
    Start(Option<T>),
    Next(T),
    End,
}

impl<T: Clone> Default for PaginationState<T> {
    fn default() -> PaginationState<T> {
        PaginationState::Start(None)
    }
}

pub struct QueryPaginator<F, T>
where
    F: Fn(&PaginationState<PaginationType>, &T) -> Option<Vec<(String, String)>>,
{
    f: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> QueryPaginator<F, T>
where
    F: Fn(&PaginationState<PaginationType>, &T) -> Option<Vec<(String, String)>>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T> Paginator<T> for QueryPaginator<F, T>
where
    F: Fn(&PaginationState<PaginationType>, &T) -> Option<Vec<(String, String)>>,
{
    fn next(
        &self,
        prev: &PaginationState<PaginationType>,
        res: &T,
    ) -> PaginationState<PaginationType> {
        let queries = (self.f)(prev, res);
        match queries {
            Some(queries) => PaginationState::Next(PaginationType::Query(queries)),
            None => PaginationState::End,
        }
    }
}
