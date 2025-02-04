use tonic::transport::Channel;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde_json::Value;
use tokio::sync::Mutex;
use std::sync::Arc;

// Include generated gRPC client
pub mod ai_review {
    include!("generated/ai_review.rs");
}

use ai_review::ai_review_client::AiReviewClient;
use ai_review::PrRequest;

#[post("/webhook")]
async fn handle_webhook(payload: web::Json<Value>, client: web::Data<Arc<Mutex<AiReviewClient<Channel>>>>) -> impl Responder {
    println!("📦 Webhook Received: {:?}", payload);

    if let Some(pull_request) = payload.get("pull_request") {
        if let (Some(repo), Some(number), Some(sha)) = (
            payload["repository"]["full_name"].as_str(),
            pull_request["number"].as_i64(),
            pull_request["head"]["sha"].as_str(),
        ) {
            println!("🔄 Sending PR #{} to AI Review Service", number);

            let mut client = client.lock().await;
            let request = tonic::Request::new(PrRequest {
                repository: repo.to_string(),
                pr_number: number,
                commit_sha: sha.to_string(),
            });

            match client.analyze_pr(request).await {
                Ok(response) => println!("✅ AI Review Summary: {}", response.into_inner().summary),
                Err(e) => {
                    eprintln!("❌ Failed to get AI Review: {}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to get AI review",
                        "details": format!("{}", e)
                    }));
                },
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({ "status": "received" }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let grpc_client = match AiReviewClient::connect("http://ai-review:50051").await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Failed to connect to AI Review Service: {}", e);
            std::process::exit(1);
        }
    };

    let shared_client = web::Data::new(Arc::new(Mutex::new(grpc_client)));

    println!("🚀 Webhook Listener running on port 3000");
    HttpServer::new(move || {
        App::new()
            .app_data(shared_client.clone())
            .service(handle_webhook)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
