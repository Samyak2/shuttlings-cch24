use salvo::Router;

mod day_1;
mod day_12;
mod day_16;
mod day_19;
mod day_2;
mod day_23;
mod day_5;
mod day_9;

pub fn get_router() -> Router {
    Router::new()
        .push(day_1::get_router())
        .push(day_2::get_router())
        .push(day_5::get_router())
        .push(day_9::get_router())
        .push(day_12::get_router())
        .push(day_16::get_router())
        .push(day_19::get_router())
        .push(day_23::get_router())
}
