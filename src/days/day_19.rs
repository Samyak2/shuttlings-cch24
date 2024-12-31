use chrono::{DateTime, Local};
use salvo::{
    oapi::{
        extract::{JsonBody, PathParam},
        BasicType, Content, Object, Schema,
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DB_POOL;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Local>,
    version: i32,
}

#[derive(Debug, thiserror::Error)]
enum QuotesError {
    #[error("database query error: {0}")]
    QueryError(#[from] sqlx::Error),

    #[error("not found")]
    NotFound,
}

impl Scribe for QuotesError {
    fn render(self, res: &mut Response) {
        match self {
            Self::QueryError(_) => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
            Self::NotFound => res.status_code(StatusCode::NOT_FOUND),
        };
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for QuotesError {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            salvo::oapi::Response::new("bad request").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            salvo::oapi::Response::new("not found").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

#[endpoint]
async fn reset_route() -> Result<&'static str, QuotesError> {
    sqlx::query("delete from quotes")
        .execute(DB_POOL.get().unwrap())
        .await?;
    Ok("")
}

#[endpoint]
async fn cite_route(id: PathParam<Uuid>) -> Result<Json<Quote>, QuotesError> {
    let quote = sqlx::query_as::<_, Quote>("select * from quotes where id = $1")
        .bind(*id)
        .fetch_optional(DB_POOL.get().unwrap())
        .await?
        .ok_or(QuotesError::NotFound)?;
    Ok(Json(quote))
}

#[endpoint]
async fn remove_route(id: PathParam<Uuid>) -> Result<Json<Quote>, QuotesError> {
    let quote = sqlx::query_as::<_, Quote>("select * from quotes where id = $1")
        .bind(*id)
        .fetch_optional(DB_POOL.get().unwrap())
        .await?
        .ok_or(QuotesError::NotFound)?;
    sqlx::query("delete from quotes where id = $1")
        .bind(*id)
        .execute(DB_POOL.get().unwrap())
        .await?;
    Ok(Json(quote))
}

#[derive(Debug, Deserialize, ToSchema)]
struct QuoteInput {
    author: String,
    quote: String,
}

#[endpoint]
async fn undo_route(
    id: PathParam<Uuid>,
    input: JsonBody<QuoteInput>,
) -> Result<Json<Quote>, QuotesError> {
    sqlx::query("update quotes set author = $1, quote = $2, version = version + 1 where id = $3")
        .bind(&input.author)
        .bind(&input.quote)
        .bind(*id)
        .execute(DB_POOL.get().unwrap())
        .await?;
    let quote = sqlx::query_as::<_, Quote>("select * from quotes where id = $1")
        .bind(*id)
        .fetch_optional(DB_POOL.get().unwrap())
        .await?
        .ok_or(QuotesError::NotFound)?;
    Ok(Json(quote))
}

#[endpoint(status_codes(201, 404, 500))]
async fn draft_route(
    input: JsonBody<QuoteInput>,
    res: &mut Response,
) -> Result<Json<Quote>, QuotesError> {
    let id = Uuid::new_v4();
    sqlx::query("insert into quotes (id, author, quote) values ($1, $2, $3) returning id")
        .bind(id)
        .bind(&input.author)
        .bind(&input.quote)
        .fetch_one(DB_POOL.get().unwrap())
        .await?;
    let quote = sqlx::query_as::<_, Quote>("select * from quotes where id = $1")
        .bind(id)
        .fetch_optional(DB_POOL.get().unwrap())
        .await?
        .ok_or(QuotesError::NotFound)?;
    res.status_code(StatusCode::CREATED);
    Ok(Json(quote))
}

pub fn get_router() -> Router {
    Router::new()
        .push(Router::with_path("/19/reset").post(reset_route))
        .push(Router::with_path("/19/cite/<id>").get(cite_route))
        .push(Router::with_path("/19/remove/<id>").delete(remove_route))
        .push(Router::with_path("/19/undo/<id>").put(undo_route))
        .push(Router::with_path("/19/draft").post(draft_route))
}
