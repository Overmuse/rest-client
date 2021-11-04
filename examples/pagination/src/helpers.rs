use super::PassengersWrapper;
use rest_client::{PaginationState, PaginationType};

fn extract_page_number(q: &PaginationType) -> Option<usize> {
    let PaginationType::Query(v) = q;
    v.first()
        .map(|(_, v)| str::parse::<usize>(v).ok())
        .flatten()
}

pub(crate) fn get_next_url(
    prev: &PaginationState<PaginationType>,
    res: &PassengersWrapper,
) -> Option<Vec<(String, String)>> {
    let max_page = res.total_pages;
    let next_page = match prev {
        PaginationState::Start(None) => Some(1),
        PaginationState::Start(Some(x)) => extract_page_number(x).map(|x| x + 1),
        PaginationState::Next(x) => extract_page_number(x).map(|x| x + 1),
        PaginationState::End => None,
    };

    next_page
        .map(|page| if page > max_page { None } else { Some(page) })
        .flatten()
        .map(|page| vec![("page".into(), format!("{}", page))])
}
