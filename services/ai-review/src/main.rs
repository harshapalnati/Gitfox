use axum::{routing::post, Router, Json};
use serde_json::Value;
use std::net::SocketAddr;
use reqwest::Client;
use tokio::task;
use std::env;
use dotenv::dotenv;
use log::{info, error};

async fn ai_review(Json(payload): Json<Value>) -> Json<Value> {
    task::spawn(async move {
        if let Some(repo) = payload["repository"].as_str() {
            if let Some(pr_number) = payload["pr_number"].as_i64() {
                if let Some(commit_sha) = payload["commit_sha"].as_str() {
                    info!("📌 Running AI Review for PR #{} in {}", pr_number, repo);
                    match analyze_pr(repo, pr_number, commit_sha).await {
                        Ok(comment) => {
                            info!("✅ AI Review Complete! Posting Comment...");
                            post_pr_comment(repo, pr_number, comment).await.ok();
                        }
                        Err(e) => error!("❌ AI Review Failed: {}", e),
                    }
                }
            }
        }
    });

    Json(serde_json::json!({ "status": "AI review triggered" }))
}

async fn analyze_pr(repo: &str, pr_number: i64, _commit_sha: &str) -> Result<String, reqwest::Error> {
    let github_token = env::var("GITHUB_TOKEN").expect("⚠️ GITHUB_TOKEN not set");
    info!("🔑 Using GitHub Token: {}", &github_token[..10]); // Partial log for security
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
                info!("📄 Analyzing {} with GPT-4...", filename);
                let ai_comment = analyze_with_gpt4(filename, patch, &openai_key).await?;
                comments.push(format!("📌 **{}**\n{}", filename, ai_comment));
            }
        }
    }

    Ok(comments.join("\n\n"))
}

async fn analyze_with_gpt4(filename: &str, code_diff: &str, openai_key: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        format!("### 📝 AI Code Review for `{}`\n", filename),
        "You are an **AI code reviewer**. Analyze the following GitHub PR change and provide a **detailed summary** with clear points.\n",
        "### 📌 Summary of Changes:",
        "- Explain what changes were made.",
        "- Highlight any **key improvements**.",
        "- Mention any **potential issues**.\n",
        "### 🔒 Security & Vulnerability Check:",
        "- Check for **security vulnerabilities**.",
        "- Identify **possible exploits or risks**.",
        "- Suggest **best security practices**.\n",
        "### 🏗️ Code Quality & Best Practices:",
        "- Detect **code smells**.",
        "- Recommend **performance improvements**.",
        "- Suggest **better coding practices**.\n",
        format!("**Code Changes in `{}`:**\n```\n{}\n```", filename, code_diff)
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

async fn post_pr_comment(repo: &str, pr_number: i64, comment: String) -> Result<(), reqwest::Error> {
    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let client = reqwest::Client::new();

    let comments_url = format!(
        "https://api.github.com/repos/{}/issues/{}/comments",
        repo, pr_number
    );

    let payload = serde_json::json!({
        "body": comment
    });

    client
        .post(&comments_url)
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "gitfox-bot")
        .json(&payload)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let app = Router::new().route("/review", post(ai_review));
    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));

    info!("🚀 AI Review Service Running on {}", addr);
    
    match env::var("GITHUB_TOKEN") {
        Ok(token) => info!("🔑 GITHUB_TOKEN Loaded: {}", &token[..10]),
        Err(_) => error!("❌ ERROR: GITHUB_TOKEN not set!"),
    }

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
