use salvo::prelude::*;

use shuttlings_cch24::days::get_router;

#[handler]
async fn hello_world(res: &mut Response) {
    res.render(Text::Plain("Hello, bird!"));
}

#[shuttle_runtime::main]
async fn salvo() -> shuttle_salvo::ShuttleSalvo {
    let router = Router::new().push(get_router()).get(hello_world);

    Ok(router.into())
}
