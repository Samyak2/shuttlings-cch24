use salvo::prelude::*;

use shuttlings_cch24::days::get_router;

#[handler]
async fn hello_world(res: &mut Response) {
    res.render(Text::Plain("Hello, bird!"));
}

#[shuttle_runtime::main]
async fn salvo() -> shuttle_salvo::ShuttleSalvo {
    let router = Router::new().push(get_router()).get(hello_world);

    let doc = OpenApi::new("Shuttle CCH 24", "0.0.1").merge_router(&router);

    let router = router
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("docs"));

    Ok(router.into())
}
