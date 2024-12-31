use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
    RsaPrivateKey, RsaPublicKey,
};
use std::{collections::HashSet, sync::LazyLock};

use salvo::{
    http::{cookie::Cookie, ParseError},
    oapi::{
        extract::CookieParam, schema::AdditionalProperties, BasicType, Content, Object,
        RequestBody, Schema,
    },
    prelude::*,
    Extractible,
};

static PRIVATE_KEY: LazyLock<RsaPrivateKey> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key")
});

static PUBLIC_KEY: LazyLock<RsaPublicKey> = LazyLock::new(|| RsaPublicKey::from(&*PRIVATE_KEY));

static ENCODING_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let key = PRIVATE_KEY.to_pkcs1_pem(LineEnding::default()).unwrap();
    EncodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});

static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let key = PUBLIC_KEY.to_pkcs1_pem(LineEnding::default()).unwrap();
    DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});

#[derive(Debug)]
struct WrapInput {
    text: Vec<u8>,
}

impl<'ex> Extractible<'ex> for WrapInput {
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

impl EndpointArgRegister for WrapInput {
    fn register(
        _components: &mut salvo::oapi::Components,
        operation: &mut salvo::oapi::Operation,
        _arg: &str,
    ) {
        operation.request_body = Some(
            RequestBody::new().description("data to wrap").add_content(
                "application/json",
                Content::new(Schema::Object(
                    Object::new()
                        .schema_type(BasicType::String)
                        .additional_properties(AdditionalProperties::FreeForm(true)),
                )),
            ),
        );
    }
}

#[derive(Debug, thiserror::Error)]
enum WrapError {
    #[error("could not parse json: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

impl Scribe for WrapError {
    fn render(self, res: &mut Response) {
        match self {
            WrapError::ParseError(_) => res.status_code(StatusCode::BAD_REQUEST),
            WrapError::JwtError(_) => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
        };
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for WrapError {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            salvo::oapi::Response::new("bad request").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            salvo::oapi::Response::new("jwt error").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

const ALGORITHM: Algorithm = Algorithm::RS256;

#[endpoint]
async fn wrap_route(data: WrapInput, res: &mut Response) -> Result<&'static str, WrapError> {
    let jsoned: serde_json::Value = serde_json::from_slice(&data.text)?;
    let encoded = encode(&Header::new(ALGORITHM), &jsoned, &ENCODING_KEY)?;
    res.add_cookie(Cookie::new("gift", encoded));
    Ok("")
}

#[endpoint]
async fn unwrap_route(gift: CookieParam<String>) -> Result<String, WrapError> {
    let mut validation = Validation::new(ALGORITHM);
    validation.required_spec_claims = HashSet::default();
    validation.validate_exp = false;
    validation.validate_aud = false;
    let token = decode::<serde_json::Value>(&gift, &DECODING_KEY, &validation)?;
    Ok(token.claims.to_string())
}

pub fn get_router() -> Router {
    Router::new()
        .push(Router::with_path("/16/wrap").post(wrap_route))
        .push(Router::with_path("/16/unwrap").get(unwrap_route))
}
