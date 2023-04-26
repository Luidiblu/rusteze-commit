use reqwest::{Client, header, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OpenAIPayload {
    prompt: String,
    max_tokens: u32,
    n: u32,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

pub async fn generate_commit_message(prompt: String, api_key: &str, api_url: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let payload = OpenAIPayload {
        prompt,
        max_tokens: 50,
        n: 1,
    };

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, format!("Bearer {}", api_key).parse()?);
    headers.insert(header::CONTENT_TYPE, "application/json".parse()?);

    let response = client.post(api_url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let openai_response = response.json::<OpenAIResponse>().await?;
    let commit_message = openai_response.choices.get(0).map(|choice| choice.text.trim().to_string());
    commit_message.ok_or_else(|| reqwest::Error::status(StatusCode::INTERNAL_SERVER_ERROR))
}
