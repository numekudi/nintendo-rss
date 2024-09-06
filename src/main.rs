use lambda_runtime::{service_fn, Error, LambdaEvent};
use reqwest::Client;
use rss::Channel;
use chrono::{DateTime, Utc, Duration};
use serde_json::{json, Value};
use std::env;
use dotenv::dotenv;



async fn handler(_event: LambdaEvent<Value>) -> Result<Value, Box<dyn std::error::Error>> {
    dotenv().ok();

    let webhook_url = env::var("WEBHOOK_URL")?;
    let rss_url = env::var("RSS_URL")?;

    let content = reqwest::get(&rss_url).await?.text().await?;

    let channel = content.parse::<Channel>()?;

    let now = Utc::now();
    let thirty_minutes_ago = now - Duration::minutes(60);

    let recent_items: Vec<_> = channel.items().iter()
        .filter(|item| {
            if let Some(pub_date) = item.pub_date() {
                if let Ok(pub_date) = DateTime::parse_from_rfc2822(pub_date) {
                    return pub_date > thirty_minutes_ago;
                }
            }
            false
        })
        .collect();

    let mut message = String::new();

    for item in recent_items {
        if let Some(title) = item.title() {
            if let Some(link) = item.link() {
                message.push_str(&format!("{}\n{}\n\n\n", title, link));
            }
        }
    }

    if !message.is_empty() {
        let client = Client::new();
        let payload = serde_json::json!({ "content": message });

        client.post(&webhook_url)
            .json(&payload)
            .send()
            .await?;
    }

    Ok(json!({"status":"ok"}))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}

