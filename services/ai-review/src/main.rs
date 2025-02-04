use tonic::{transport::Server, Request, Response, Status};
use ai_review::ai_review_server::{AiReview, AiReviewServer};
use ai_review::{PrRequest, PrResponse};
use tokio::sync::Mutex;
use std::sync::Arc;
use reqwest::Client;
use std::env;
use serde_json::Value;

pub mod ai_review {
    include!("generated/ai_review.rs");  // Load gRPC protobuf files
}

#[derive(Debug, Default)]
pub struct AiReviewService {
    pub pr_count: Arc<Mutex<u64>>,  // Track PRs reviewed
}

#[tonic::async_trait]
impl AiReview for AiReviewService {
    async fn analyze_pr(&self, request: Request<PrRequest>) -> Result<Response<PrResponse>, Status> {
        let req = request.into_inner();
        println!("📌 Running AI Review for PR #{} in {}", req.pr_number, req.repository);
    
        // **🔹 Step 1: Mark PR as "pending" immediately**
        if let Err(e) = set_github_pr_status(&req.repository, &req.commit_sha, "pending", "AI Review in progress").await {
            eprintln!("⚠️ Failed to set pending status on GitHub: {}", e);
        }
    
        // **🔹 Step 2: Fetch AI Review from OpenAI**
        match get_openai_analysis(&req.repository, req.pr_number, &req.commit_sha).await {
            Ok(summary) => {
                println!("✅ AI Review Summary Generated: {}", summary);
    
                // **🔹 Step 3: Post AI review comment**
                if let Err(e) = post_github_pr_comment(&req.repository, req.pr_number, &summary).await {
                    eprintln!("⚠️ Failed to post AI review comment: {}", e);
                }
    
                // **🔹 Step 4: Mark PR as "success" to allow merging**
                if let Err(e) = set_github_pr_status(&req.repository, &req.commit_sha, "success", "AI Review completed").await {
                    eprintln!("⚠️ Failed to update GitHub PR status: {}", e);
                }
    
                Ok(Response::new(PrResponse { summary }))
            },
            Err(e) => {
                eprintln!("❌ OpenAI Analysis Failed: {}", e);
                // Mark PR as "failure" if AI review fails
                set_github_pr_status(&req.repository, &req.commit_sha, "failure", "AI Review failed").await.ok();
                Err(Status::internal("Failed to generate AI review"))
            }
        }
    }
    
}

async fn get_openai_analysis(repo: &str, pr_number: i64, commit_sha: &str) -> Result<String, reqwest::Error> {
    let github_token = env::var("GITHUB_TOKEN").expect("⚠️ GITHUB_TOKEN not set");
    log::info!("🔑 Using GitHub Token: {}", &github_token[..10]); // Partial log for security
    let openai_key = env::var("OPENAI_API_KEY").expect("⚠️ OPENAI_API_KEY not set");

    let client = Client::new();
    let pr_url = format!("https://api.github.com/repos/{}/pulls/{}/files", repo, pr_number);

    let pr_response = client
        .get(&pr_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "gitfox-bot")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let mut comments = vec![];

    for file in pr_response.as_array().unwrap_or(&vec![]) {
        if let Some(filename) = file["filename"].as_str() {
            if let Some(patch) = file["patch"].as_str() {
                log::info!("📄 Analyzing {} with GPT-4...", filename);
                let ai_comment = analyze_with_gpt4(filename, patch, &openai_key).await?;
                println!("{}", ai_comment);
                comments.push(format!("📌 **{}**\n{}", filename, ai_comment));

            }
        }
    }

    Ok(comments.join("\n\n"))
}


async fn analyze_with_gpt4(filename: &str, code_diff: &str, openai_key: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        format!("### 📝 AI Code Review for `{}`", filename),
        "",
        "You are an **AI code reviewer**. Analyze the following GitHub PR change and provide a **detailed summary** with clear points.",
        "",
        "### 📌 Summary of Changes:",
        "- Explain what changes were made.",
        "- Highlight any **key improvements**.",
        "- Mention any **potential issues**.",
        "",
        "### 🔒 Security & Vulnerability Check:",
        "- Check for **security vulnerabilities**.",
        "- Identify **possible exploits or risks**.",
        "- Suggest **best security practices**.",
        "",
        "### 🏗️ Code Quality & Best Practices:",
        "- Detect **code smells**.",
        "- Recommend **performance improvements**.",
        "- Suggest **better coding practices**.",
        "",
        format!("**Code Changes in `{}`:**", filename),
        format!("```\n{}\n```", code_diff)
    );

    let payload = serde_json::json!({
        "model": "gpt-4",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 500,
        "temperature": 0.5
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", openai_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No AI feedback available.")
        .to_string())
}


async fn post_github_pr_comment(repo: &str, pr_number: i64, comment: &str) -> Result<(), reqwest::Error> {
    let github_token = env::var("GITHUB_TOKEN").expect("❌ GITHUB_TOKEN not set!");

    let client = Client::new();
    let comments_url = format!("https://api.github.com/repos/{}/issues/{}/comments", repo, pr_number);

    let payload = serde_json::json!({
        "body": format!("### 🤖 AI Code Review\n\n{}", comment)
    });

    let _response = client
        .post(&comments_url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "GitFox-AI-Review/1.0")
        .json(&payload)
        .send()
        .await?;

    println!("✅ AI Review Comment Posted to PR #{}", pr_number);

    Ok(())
}

async fn set_github_pr_status(repo: &str, commit_sha: &str, state: &str, description: &str) -> Result<(), reqwest::Error> {
    let github_token = match env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("⚠️ GITHUB_TOKEN not set! Skipping GitHub status update.");
            return Ok(());
        }
    };

    let client = Client::new();
    let status_url = format!("https://api.github.com/repos/{}/statuses/{}", repo, commit_sha);

    let payload = serde_json::json!({
        "state": state,  // "pending", "success", or "failure"
        "description": description,
        "context": "GitFox AI Review",  // **GitHub uses this to track required checks**
    });

    let response = client
        .post(&status_url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "GitFox-AI-Review/1.0")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("✅ GitHub status updated successfully: {}", state);
    } else {
        eprintln!("❌ GitHub API error: {}", response.text().await?);
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::0]:50051".parse()?;
    let ai_review = AiReviewService::default();

    println!("🚀 AI Review gRPC Service Running on {}", addr);
    Server::builder()
        .add_service(AiReviewServer::new(ai_review))
        .serve(addr)
        .await?;

    Ok(())
}
