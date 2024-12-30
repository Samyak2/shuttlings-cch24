use cargo_toml::Manifest;
use salvo::{
    http::ParseError,
    oapi::{BasicType, Content, Object, RequestBody, Schema},
    prelude::*,
    Extractible,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Order {
    pub item: String,
    pub quantity: usize,
}

#[derive(Debug)]
struct ManifestInput {
    text: Vec<u8>,
}

impl<'ex> Extractible<'ex> for ManifestInput {
    fn metadata() -> &'ex salvo::extract::Metadata {
        static METADATA: salvo::extract::Metadata = salvo::extract::Metadata::new("");
        &METADATA
    }

    async fn extract(
        req: &'ex mut Request,
    ) -> Result<Self, impl Writer + Send + std::fmt::Debug + 'static> {
        Ok::<Self, ParseError>(Self {
            text: req.payload().await?.into_iter().cloned().collect(),
        })
    }
}

impl EndpointArgRegister for ManifestInput {
    fn register(
        _components: &mut salvo::oapi::Components,
        operation: &mut salvo::oapi::Operation,
        _arg: &str,
    ) {
        operation.request_body = Some(
            RequestBody::new()
                .description("Manifest as TOML")
                .add_content(
                    "text/plain",
                    Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
                ),
        );
    }
}

#[derive(Debug, thiserror::Error)]
enum Day5Error {
    #[error("Invalid manifest")]
    ParseError(#[from] cargo_toml::Error),

    #[error("Magic keyword not provided")]
    NoMagicKeyword,

    #[error("no orders")]
    NoOrders,
}

#[async_trait]
impl Writer for Day5Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::ParseError(_) | Self::NoMagicKeyword => {
                res.status_code(StatusCode::BAD_REQUEST);
            }
            Self::NoOrders => {
                res.status_code(StatusCode::NO_CONTENT);
            }
        }
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for Day5Error {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::NO_CONTENT.as_str(),
            salvo::oapi::Response::new("Orders not found")
                .add_content("text/plain", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            salvo::oapi::Response::new("Invalid Manifest or no magic keyword")
                .add_content("text/plain", StatusError::to_schema(components)),
        );
    }
}

#[endpoint]
async fn manifest_route(data: ManifestInput) -> Result<String, Day5Error> {
    let manifest = Manifest::from_slice(&data.text)?;
    let package = manifest.package.unwrap();

    if !package
        .keywords
        .get()
        .map_err(|_| Day5Error::NoOrders)?
        .iter()
        .any(|s| s == "Christmas 2024")
    {
        return Err(Day5Error::NoMagicKeyword);
    }

    let metadata = package.metadata.ok_or(Day5Error::NoOrders)?;
    let orders = metadata
        .get("orders")
        .ok_or(Day5Error::NoOrders)?
        .as_array()
        .ok_or(Day5Error::NoOrders)?;

    let outputs: Vec<_> = orders
        .iter()
        .filter_map(|order| {
            let order: Result<Order, _> = order.clone().try_into();
            if let Ok(order) = order {
                Some(format!("{}: {}", order.item, order.quantity))
            } else {
                None
            }
        })
        .collect();

    if outputs.is_empty() {
        Err(Day5Error::NoOrders)
    } else {
        Ok(outputs.join("\n"))
    }
}

pub fn get_router() -> Router {
    Router::new().push(Router::with_path("/5/manifest").post(manifest_route))
}
