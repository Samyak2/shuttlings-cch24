use std::net::Ipv4Addr;

use salvo::{oapi::extract::QueryParam, prelude::*};

#[endpoint]
async fn dest(from: QueryParam<Ipv4Addr, true>, key: QueryParam<Ipv4Addr, true>) -> String {
    let [from_octet_1, from_octet_2, from_octet_3, from_octet_4] = from.octets();
    let [key_octet_1, key_octet_2, key_octet_3, key_octet_4] = key.octets();
    let result = Ipv4Addr::new(
        from_octet_1.wrapping_add(key_octet_1),
        from_octet_2.wrapping_add(key_octet_2),
        from_octet_3.wrapping_add(key_octet_3),
        from_octet_4.wrapping_add(key_octet_4),
    );
    result.to_string()
}

#[endpoint]
async fn key_(from: QueryParam<Ipv4Addr, true>, to: QueryParam<Ipv4Addr, true>) -> String {
    let [from_octet_1, from_octet_2, from_octet_3, from_octet_4] = from.octets();
    let [to_octet_1, to_octet_2, to_octet_3, to_octet_4] = to.octets();
    let result = Ipv4Addr::new(
        to_octet_1.wrapping_sub(from_octet_1),
        to_octet_2.wrapping_sub(from_octet_2),
        to_octet_3.wrapping_sub(from_octet_3),
        to_octet_4.wrapping_sub(from_octet_4),
    );
    result.to_string()
}

pub fn get_router() -> Router {
    Router::new()
        .push(Router::with_path("/2/dest").get(dest))
        .push(Router::with_path("/2/key").get(key_))
}
