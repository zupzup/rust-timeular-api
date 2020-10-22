use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs;

type Error = Box<dyn std::error::Error>;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

const BASE_URL: &str = "https://api.timeular.com/api/v3";
const REPORT_FILE: &str = "./report.csv";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = env::var("TMLR_API_KEY").expect("TMLR_API_KEY needs to be set");
    let api_secret = env::var("TMLR_API_SECRET").expect("TMLR_API_SECRET needs to be set");

    println!("signing in..");
    let token = sign_in(api_key, api_secret).await?;

    println!("fetching me and spaces...");
    let me = fetch_me(&token).await?;
    let spaces = fetch_spaces(&token).await?;
    println!("fetched spaces: {:?} for {:?}", spaces, me);

    println!("fetching activities...");
    let activities = fetch_activities(&token).await?;
    println!("activities: {:?}", activities);

    if !activities.is_empty() {
        println!("starting to track...");
        let tracking = start_tracking(&activities.get(0).expect("exists").id, &token).await?;
        println!("started tracking: {:?}", tracking);
        let time_entry = stop_tracking(&token).await?;
        println!("created time entry: {:?}", time_entry);
    }

    println!("creating report...");
    generate_report(&token).await?;
    println!("downloaded report to: {}", REPORT_FILE);

    Ok(())
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

#[derive(Deserialize, Debug)]
struct MeResponse {
    data: Me,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Me {
    user_id: String,
    name: String,
    email: String,
    default_space_id: String,
}

async fn fetch_me(token: &str) -> Result<Me, Error> {
    let resp = CLIENT
        .get(&url("/me"))
        .header("Authorization", auth(token))
        .send()
        .await?
        .json::<MeResponse>()
        .await?;
    Ok(resp.data)
}

#[derive(Deserialize, Debug)]
struct SpacesResponse {
    data: Vec<Space>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Space {
    id: String,
    name: String,
    default: bool,
    members: Vec<Member>,
    retired_members: Vec<RetiredMember>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Member {
    id: String,
    name: String,
    email: String,
    role: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RetiredMember {
    id: String,
    name: String,
}

async fn fetch_spaces(token: &str) -> Result<Vec<Space>, Error> {
    let resp = CLIENT
        .get(&url("/space"))
        .header("Authorization", auth(token))
        .send()
        .await?
        .json::<SpacesResponse>()
        .await?;
    Ok(resp.data)
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TrackingRequest {
    started_at: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TrackingResponse {
    current_tracking: Tracking,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Tracking {
    id: i64,
    activity_id: String,
    started_at: String,
    note: Note,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Note {
    text: Option<String>,
    tags: Vec<TagOrMention>,
    mentions: Vec<TagOrMention>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TagOrMention {
    id: i64,
    key: String,
    label: String,
    scope: String,
    space_id: String,
}

async fn start_tracking(activity_id: &str, token: &str) -> Result<Tracking, Error> {
    let body = TrackingRequest {
        started_at: "2020-08-03T04:00:00.000".to_string(),
    };
    let resp = CLIENT
        .post(&url(&format!("/tracking/{}/start", activity_id)))
        .header("Authorization", auth(token))
        .json(&body)
        .send()
        .await?
        .json::<TrackingResponse>()
        .await?;
    Ok(resp.current_tracking)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct StopTrackingRequest {
    stopped_at: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TimeEntryResponse {
    created_time_entry: TimeEntry,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TimeEntry {
    id: String,
    activity_id: String,
    duration: Duration,
    note: Note,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Duration {
    started_at: String,
    stopped_at: String,
}

async fn stop_tracking(token: &str) -> Result<TimeEntry, Error> {
    let body = StopTrackingRequest {
        stopped_at: "2020-08-03T05:00:00.000".to_string(),
    };
    let resp = CLIENT
        .post(&url("/tracking/stop"))
        .header("Authorization", auth(token))
        .json(&body)
        .send()
        .await?
        .json::<TimeEntryResponse>()
        .await?;
    Ok(resp.created_time_entry)
}

async fn generate_report(token: &str) -> Result<(), Error> {
    let resp = CLIENT
        .get(&url(
            "/report/2020-01-01T00:00:00.000/2020-12-31T23:59:59.999?timezone=Europe/Vienna",
        ))
        .header("Authorization", auth(token))
        .send()
        .await?
        .bytes()
        .await?;
    fs::write(REPORT_FILE, &resp).await?;
    Ok(())
}

fn url(path: &str) -> String {
    format!("{}{}", BASE_URL, path)
}

fn auth(token: &str) -> String {
    format!("Bearer {}", token)
}
