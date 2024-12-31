use std::{fmt::Display, str::FromStr};

use salvo::{
    oapi::{extract::PathParam, BasicType, Content, Object, Schema},
    prelude::*,
};

#[derive(Debug)]
struct Html {
    text: String,
}

impl Html {
    fn new(text: String) -> Self {
        Self { text }
    }
}

impl Scribe for Html {
    fn render(self, res: &mut Response) {
        res.render(Text::Html(self.text));
    }
}

impl EndpointOutRegister for Html {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::OK.as_str(),
            salvo::oapi::Response::new("ok").add_content(
                "text/html",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

#[endpoint]
async fn star_route() -> Html {
    Html::new(r#"<div id="star" class="lit"></div>"#.to_owned())
}

#[derive(Debug)]
enum Color {
    Red,
    Blue,
    Purple,
}

impl Color {
    pub fn next(&self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Purple,
            Self::Purple => Self::Red,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Red => "red",
                Self::Blue => "blue",
                Self::Purple => "purple",
            }
        )
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Self::Red),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            _ => Err(()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("invalid")]
    Invalid,
}

impl Scribe for Error {
    fn render(self, res: &mut Response) {
        match self {
            Self::Invalid => res.status_code(StatusCode::IM_A_TEAPOT),
        };
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for Error {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::IM_A_TEAPOT.as_str(),
            salvo::oapi::Response::new("i am a teapot").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

#[endpoint]
async fn present_color_route(color: PathParam<String>) -> Result<Html, Error> {
    let color: Color = color.parse().map_err(|_| Error::Invalid)?;
    Ok(Html::new(format!(
        r#"<div
            class="present {color}"
            hx-get="/23/present/{}"
            hx-swap="outerHTML"
        >
            <div class="ribbon"></div>
            <div class="ribbon"></div>
            <div class="ribbon"></div>
            <div class="ribbon"></div>
        </div>"#,
        color.next()
    )))
}

#[derive(Debug)]
enum State {
    On,
    Off,
}

impl State {
    pub fn next(&self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::On => "on",
                Self::Off => "off",
            }
        )
    }
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(()),
        }
    }
}

#[endpoint]
async fn ornament_route(state: PathParam<String>, n: PathParam<String>) -> Result<Html, Error> {
    let state: State = state.parse().map_err(|_| Error::Invalid)?;
    let n = html_escape::encode_double_quoted_attribute(&*n);
    let next_state = state.next();
    let on_class = match state {
        State::On => " on",
        State::Off => "",
    };
    Ok(Html::new(format!(
        r#"<div
            class="ornament{on_class}"
            id="ornament{n}"
            hx-trigger="load delay:2s once"
            hx-get="/23/ornament/{next_state}/{n}"
            hx-swap="outerHTML"
        ></div>"#,
    )))
}

pub fn get_router() -> Router {
    Router::new()
        .push(Router::with_path("/23/star").get(star_route))
        .push(Router::with_path("/23/present/<color>").get(present_color_route))
        .push(Router::with_path("/23/ornament/<state>/<n>").get(ornament_route))
}
