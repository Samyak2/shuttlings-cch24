use salvo::logging::Logger;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;

use shuttlings_cch24::days::get_router;
use shuttlings_cch24::db::DB_POOL;

#[handler]
async fn hello_world(res: &mut Response) {
    res.render(Text::Plain("Hello, bird!"));
}

#[shuttle_runtime::main]
async fn salvo(#[shuttle_shared_db::Postgres] pool: sqlx::PgPool) -> shuttle_salvo::ShuttleSalvo {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    DB_POOL.set(pool).expect("could not set DB_POOL");

    let router = Router::new()
        .push(get_router())
        .get(hello_world)
        .push(Router::with_path("/assets/<**path>").get(StaticDir::new(["assets/"])));

    let doc = OpenApi::new("Shuttle CCH 24", "0.0.1").merge_router(&router);

    let router = router
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("docs"))
        .hoop(Logger::new());

    Ok(router.into())
}
