use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use salvo::{
    http::ParseError,
    oapi::{schema::OneOf, BasicType, Content, Object, RequestBody, Schema},
    prelude::*,
    Extractible,
};
use serde::{Deserialize, Serialize};

static COUNTER: AtomicU64 = AtomicU64::new(5);

fn saturating_dec_atm(val: &AtomicU64) -> u64 {
    loop {
        let current = val.load(Ordering::Relaxed);
        if current > 0 {
            if let Ok(current) =
                val.compare_exchange(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
            {
                // successfully decremented
                return current;
            }
            // decrement did not succeed. try operation from beginning again.
        } else {
            // is 0 already. can't subtract more.
            return 0;
        }
    }
}

fn saturating_inc_atm(val: &AtomicU64, max: u64) -> u64 {
    loop {
        let current = val.load(Ordering::Relaxed);
        if current < max {
            if let Ok(current) =
                val.compare_exchange(current, current + 1, Ordering::Relaxed, Ordering::Relaxed)
            {
                // successfully incremented
                return current;
            }
            // incremented did not succeed. try operation from beginning again.
        } else {
            // is max already. can't subtract more.
            return max;
        }
    }
}

#[handler]
async fn rate_limit_middleware(res: &mut Response) {
    if saturating_dec_atm(&COUNTER) == 0 {
        res.status_code(StatusCode::TOO_MANY_REQUESTS);
        res.render(Text::Plain("No milk available\n"));
        return;
    }
}

#[derive(Debug, ToSchema)]
enum MilkInput {
    Empty,
    Convert(ConvertInput),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct Liters {
    pub liters: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct Litres {
    pub litres: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct Pints {
    pub pints: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
struct Gallons {
    pub gallons: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
enum ConvertInput {
    Liters(Liters),
    Gallons(Gallons),
    Litres(Litres),
    Pints(Pints),
}

impl<'ex> Extractible<'ex> for MilkInput {
    fn metadata() -> &'ex salvo::extract::Metadata {
        static METADATA: salvo::extract::Metadata = salvo::extract::Metadata::new("");
        &METADATA
    }

    async fn extract(
        req: &'ex mut Request,
    ) -> Result<Self, impl Writer + Send + std::fmt::Debug + 'static> {
        match req.content_type() {
            Some(mime) if mime.essence_str() == "application/json" => {
                let bruh: ConvertInput = req.parse_json::<ConvertInput>().await?;
                Ok::<Self, Day9Error>(Self::Convert(bruh))
            }
            _ => Ok(Self::Empty),
        }
    }
}

impl EndpointArgRegister for MilkInput {
    fn register(
        components: &mut salvo::oapi::Components,
        operation: &mut salvo::oapi::Operation,
        _arg: &str,
    ) {
        operation.request_body = Some(
            RequestBody::new()
                .add_content(
                    "",
                    Content::new(Schema::Object(Object::new().schema_type(BasicType::Null))),
                )
                .add_content(
                    "application/json",
                    Content::new(ConvertInput::to_schema(components)),
                ),
        );
    }
}

#[derive(Debug, thiserror::Error)]
enum Day9Error {
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
}

#[async_trait]
impl Writer for Day9Error {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::ParseError(_) => {
                res.status_code(StatusCode::BAD_REQUEST);
            }
        }
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for Day9Error {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::TOO_MANY_REQUESTS.as_str(),
            salvo::oapi::Response::new("no milk").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            salvo::oapi::Response::new("no milk").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

#[derive(Debug, ToSchema)]
enum MilkOutput {
    String(String),
    Convert(ConvertInput),
}

#[async_trait]
impl Writer for MilkOutput {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::String(s) => {
                res.render(Text::Plain(s));
            }
            Self::Convert(convert) => {
                res.render(Json(convert));
            }
        }
    }
}

impl EndpointOutRegister for MilkOutput {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::OK.as_str(),
            salvo::oapi::Response::new("success").add_content(
                "text/plain",
                Content::new(Schema::OneOf(
                    OneOf::new()
                        .item(Schema::Object(Object::new().schema_type(BasicType::String)))
                        .item(ConvertInput::to_schema(components)),
                )),
            ),
        );
    }
}

const GALLON_TO_LITER: f64 = 3.78541;
const PINT_TO_LITRE: f64 = 0.568261;

#[endpoint]
async fn milk_route(inputs: MilkInput) -> Result<MilkOutput, Day9Error> {
    match inputs {
        MilkInput::Empty => Ok(MilkOutput::String("Milk withdrawn\n".to_owned())),
        MilkInput::Convert(ConvertInput::Liters(Liters { liters })) => {
            Ok(MilkOutput::Convert(ConvertInput::Gallons(Gallons {
                gallons: liters / GALLON_TO_LITER,
            })))
        }
        MilkInput::Convert(ConvertInput::Gallons(Gallons { gallons })) => {
            Ok(MilkOutput::Convert(ConvertInput::Liters(Liters {
                liters: gallons * GALLON_TO_LITER,
            })))
        }
        MilkInput::Convert(ConvertInput::Litres(Litres { litres })) => {
            Ok(MilkOutput::Convert(ConvertInput::Pints(Pints {
                pints: litres / PINT_TO_LITRE,
            })))
        }
        MilkInput::Convert(ConvertInput::Pints(Pints { pints })) => {
            Ok(MilkOutput::Convert(ConvertInput::Litres(Litres {
                litres: pints * PINT_TO_LITRE,
            })))
        }
    }
}

pub fn get_router() -> Router {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            saturating_inc_atm(&COUNTER, 5);
        }
    });
    Router::new()
        .hoop(rate_limit_middleware)
        .push(Router::with_path("/9/milk").post(milk_route))
}
