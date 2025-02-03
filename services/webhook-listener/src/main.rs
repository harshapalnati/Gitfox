use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde_json::Value;
use dotenv::dotenv;
use env_logger;

#[post("/webhook")]
async fn handle_webhook(payload: web::Json<Value>) -> impl Responder {
    println!("📦 Webhook Received: {:?}", payload);

    // Extract PR details
    if let Some(pull_request) = payload.get("pull_request") {
        if let (Some(repo), Some(number), Some(sha)) = (
            payload["repository"]["full_name"].as_str(),
            pull_request["number"].as_i64(),
            pull_request["head"]["sha"].as_str(),
        ) {
            println!("🔄 Processing PR #{} in {}", number, repo);

            // Send to AI Review Service
            match send_to_ai_review(repo, number, sha).await {
                Ok(_) => println!("✅ Successfully sent to AI Review Service"),
                Err(e) => eprintln!("❌ Failed to send to AI Review: {}", e),
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "received"
    }))
}

async fn send_to_ai_review(repo: &str, pr_number: i64, pr_sha: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Get the AI review service URL from environment variable, or use default
    let ai_review_url = std::env::var("AI_REVIEW_URL")
        .unwrap_or_else(|_| "http://localhost:5000/review".to_string());

    println!("🔄 Sending request to AI Review Service at: {}", ai_review_url);

    let payload = serde_json::json!({
        "repository": repo,
        "pr_number": pr_number,
        "commit_sha": pr_sha
    });

    let response = client
        .post(&ai_review_url)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10)) // Add timeout
        .send()
        .await?;

    println!("✅ AI Review Service Response Status: {}", response.status());

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize logger
    env_logger::init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    println!("🚀 Webhook Listener running on port {}", port);

    HttpServer::new(|| {
        App::new()
            .service(handle_webhook)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
