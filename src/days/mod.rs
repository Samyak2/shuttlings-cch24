use salvo::Router;

mod day_1;

pub fn get_router() -> Router {
    Router::new().push(day_1::get_router())
}
