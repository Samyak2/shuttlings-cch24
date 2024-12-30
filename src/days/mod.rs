use salvo::Router;

mod day_1;
mod day_2;
mod day_5;

pub fn get_router() -> Router {
    Router::new()
        .push(day_1::get_router())
        .push(day_2::get_router())
        .push(day_5::get_router())
}
