# GitFox Documentation

## 1. Project Overview
GitFox is an AI-powered GitHub code review bot that automates pull request analysis, detects bugs, and provides inline feedback using AI models like GPT-4 and CodeBERT. The tool integrates seamlessly with GitHub repositories, offering automated review suggestions and improving code quality.

---
## 2. Installation and Setup
### Prerequisites
Ensure you have the following installed on your system:
- **Docker** (for containerization)
- **ngrok** (for local webhook testing)
- **GitHub App Registration** (for GitHub integration)

### Clone the Repository
```sh
git clone https://github.com/your-org/gitfox.git
cd gitfox
```

### Run All Services with Docker
```sh
docker-compose up --build
```

---
## 3. Setting Up Git Branch Rules
To maintain clean code contributions, enforce the following branch rules in GitHub:
- **Main Branch (protected)**: Only merge after CI/CD checks pass.
- **Feature Branches**: Branch off `main`, use naming conventions like `feature/<name>`.
- **Pull Requests**:
  - Require at least 1 review before merging.
  - Ensure all tests pass before merging.
  - Squash commits before merging.

### Enforcing Branch Rules
1. Go to **GitHub → Repo → Settings → Branches**.
2. Click **Add branch protection rule**.
3. Enable **Require pull request reviews before merging**.
4. Enable **Require status checks to pass before merging**.

---
## 4. Setting Up ngrok for Local Webhook Testing
To test webhook events locally, set up `ngrok`:

### Step 1: Start ngrok
```sh
grok http 8080
```
This will generate a public URL, e.g., `https://xyz.ngrok.io`

### Step 2: Update Webhook URL in GitHub
1. Go to **GitHub → Settings → Webhooks**.
2. Click **Add webhook**.
3. Enter the `ngrok` URL (`https://xyz.ngrok.io`) with `/webhook` endpoint.
4. Set **Content Type** to `application/json`.
5. Click **Save**.

---
## 5. API Documentation
The GitFox API supports the following endpoints:

### **Webhook Listener**
```http
POST /webhook
```
Receives GitHub events and processes PRs.

### **AI Review Service**
```http
POST /ai/review
```
Runs AI-powered analysis on a pull request.

### **GitHub Integration**
```http
GET /github/repos
```
Fetches repositories linked to the GitFox bot.

---
## 6. Contribution Guidelines
To contribute:
1. Fork the repo.
2. Create a feature branch (`feature/<your-feature>`).
3. Make changes and commit.
4. Submit a pull request.

---
## 7. License
GitFox is licensed under MIT License. See `LICENSE.md` for details.

For further details, refer to individual service documentation inside the `docs/` folder.

