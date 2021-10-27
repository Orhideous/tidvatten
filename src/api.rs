#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use regex::Regex;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{json::Json, Deserialize, Serialize};

/// Single seeded release
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SeededRelease<'a> {
    /// Release identifier, corresponds to topic ID on forum
    id: u32,
    /// Torrent hash
    hash: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ReportRequest<'a> {
    /// All keeped releases
    #[serde(borrow)]
    releases: Vec<SeededRelease<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct ReportResponse<'a> {
    timestamp: DateTime<Utc>,
    message: &'a str,
}

#[derive(Debug)]
struct AuthenticatedUser<'a> {
    username: &'a str,
    token: &'a str,
}

#[derive(Debug)]
enum LoginError {
    Missing,
    Invalid,
    UserDoesNotExist,
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"Token\s(.*)").unwrap();
}

fn is_valid(key: &str) -> bool {
    RE.is_match(key)
}

async fn extract_user(authorization: &str) -> Result<AuthenticatedUser<'_>, LoginError> {
    match RE
        .captures(authorization)
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str())
    {
        None => Err(LoginError::Invalid),
        Some(keeper_api_key) => {
            // TODO: Fetch real API here
            Ok(AuthenticatedUser {
                username: "test",
                token: keeper_api_key,
            })
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthenticatedUser<'a> {
    type Error = LoginError;

    async fn from_request(req: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            None => {
                warn!("Received request without authorization header!");
                Outcome::Failure((Status::BadRequest, LoginError::Missing))
            }
            Some(token) if is_valid(token) => {
                debug!("Received valid token {}, extracting user…", token);
                let user = extract_user(token).await;
                user.into_outcome(Status::Unauthorized)
            }
            Some(token) => {
                warn!("Recived invalid token: {}", token);
                Outcome::Failure((Status::BadRequest, LoginError::Invalid))
            }
        }
    }
}

#[post("/report", data = "<report>", format = "application/json")]
fn report<'a>(
    report: Json<ReportRequest>,
    user: AuthenticatedUser<'a>,
) -> Json<ReportResponse<'a>> {
    let utc: DateTime<Utc> = Utc::now();
    info!(
        "Received report with {} releases from user {}",
        report.releases.len(),
        user.username
    );
    Json(ReportResponse {
        timestamp: utc,
        message: "Report enqueued",
    })
}

#[catch(default)]
fn default_catcher<'a>(status: Status, _request: &'a Request) -> Json<ReportResponse<'a>> {
    Json(ReportResponse {
        timestamp: Utc::now(),
        message: status.reason().unwrap_or("Unknown error"),
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![report]
}

pub fn catchers() -> Vec<rocket::Catcher> {
    catchers![default_catcher]
}
