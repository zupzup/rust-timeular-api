#[macro_use]
extern crate lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

type Error = Box<dyn std::error::Error>;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

const BASE_URL: &str = "https://api.timeular.com/api/v3";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = env::var("TMLR_API_KEY").expect("TMLR_API_KEY needs to be set");
    let api_secret = env::var("TMLR_API_SECRET").expect("TMLR_API_SECRET needs to be set");

    println!("signing in..");
    let token = sign_in(api_key, api_secret).await?;
    // TODO: get /me
    // TODO: get my spaces
    println!("fetching activities...");
    let activities = fetch_activities(&token).await?;
    // TODO: create time entry
    // TODO: start tracking, cancel tracking
    // TODO: create and write excel report to disk

    println!("activities: {:?}", activities);

    Ok(())
}

async fn sign_in(api_key: String, api_secret: String) -> Result<String, Error> {
    let body = SignInRequest {
        api_key,
        api_secret,
    };
    let resp = CLIENT
        .post(&url("/developer/sign-in"))
        .json(&body)
        .send()
        .await?
        .json::<SignInResponse>()
        .await?;
    Ok(resp.token)
}

async fn fetch_activities(token: &str) -> Result<Vec<Activity>, Error> {
    let resp = CLIENT
        .get(&url("/activities"))
        .header("Authorization", auth(token))
        .send()
        .await?
        .json::<ActivitiesResponse>()
        .await?;
    Ok(resp.activities)
}

fn url(path: &str) -> String {
    format!("{}{}", BASE_URL, path)
}

fn auth(token: &str) -> String {
    format!("Bearer {}", token)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignInRequest {
    api_key: String,
    api_secret: String,
}

#[derive(Deserialize, Debug)]
struct SignInResponse {
    token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ActivitiesResponse {
    activities: Vec<Activity>,
    inactive_activities: Vec<Activity>,
    archived_activities: Vec<Activity>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Activity {
    id: String,
    name: String,
    color: String,
    integration: String,
    space_id: String,
    device_side: Option<i64>,
}
