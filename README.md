# GitFox Documentation

## 1. Project Overview
GitFox is an AI-powered GitHub code review bot that automates pull request analysis, detects bugs, and provides inline feedback using AI models like GPT-4 and CodeBERT. The tool integrates seamlessly with GitHub repositories, offering automated review suggestions and improving code quality.

GitFox is an open-source project. It is strictly not for sale or commercial use. Any attempts to monetize or sell GitFox violate its intended purpose.

Disclaimer: GitFox is an independently developed open-source project and is not affiliated with, endorsed by, or associated with any other similar platform.  This project is released under the Apache License, and it is the responsibility of contributors and users to ensure compliance with relevant laws and regulations.

### **Data Collection Disclaimer**
GitFox collects **non-sensitive usage statistics**, such as the number of PRs reviewed and unique repositories using GitFox. This data is used to improve the project and assist with funding efforts. No personal or sensitive data is collected.


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

### Create a `.env` File
GitFox requires environment variables for configuration. Create a `.env` file in the project root and define the required values:

create .env file in root and in the infra directory
```ini
GITHUB_TOKEN=your_github_token
OPENAI_API_KEY=your_openai_api_key
```
> **Note:** Ensure the `.env` file is **not committed** to your repository. Add it to `.gitignore`.


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
1. Go to **GitHub → Repo → Settings → Rules**.
2. Click **Add branch protection rule**.
![alt text](image.png)
![alt text](image-1.png)
3. Enable **Require pull request reviews before merging**.
4. Enable **Require status checks to pass before merging**.
![alt text](image-2.png)

---
## 4. Setting Up ngrok for Local Webhook Testing
To test webhook events locally, set up `ngrok`:
https://dashboard.ngrok.com/get-started/setup/windows

### Step 1: Start ngrok
```sh
ngrok http http://localhost:3000 
```
This will generate a public URL, e.g., `https://xyz.ngrok.io`
![alt text](image-3.png)
### Step 2: Update Webhook URL in GitHub
1. Go to **GitHub → Settings → Webhooks**.
2. Click **Add webhook**.
3. Enter the `ngrok` URL (`https://xyz.ngrok.io`) with `/webhook` endpoint.
4. Set **Content Type** to `application/json`.
5. Click **Save**.
![alt text](image-4.png)
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

