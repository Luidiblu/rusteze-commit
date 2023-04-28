use git2::{Repository, Signature};
use clap::{App, Arg};
use reqwest;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("http error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("git error: {0}")]
    GitError(#[from] git2::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("API error: {0}")]
    ApiError(String),
}

async fn get_commit_message(api_key: &str, prompt: &str) -> Result<String, MyError> {
    let client = reqwest::Client::new();
    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}]
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;

    if let Some(message) = json["choices"].get(0).and_then(|choice| choice["text"].as_str()) {
        Ok(message.trim().to_string())
    } else {
        Err(MyError::ApiError(format!("Failed to extract commit message from API response: {:?}", json)))
    }
}

fn get_diff(repo: &Repository) -> Result<String, MyError> {
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;
    let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), None)?;

    let mut diff_str = String::new();
    for delta in diff.deltas() {
        let file_path = delta.new_file().path().unwrap().to_str().unwrap();
        diff_str.push_str(file_path);
        diff_str.push('\n');
    }

    Ok(diff_str)
}

fn create_commit(repo: &Repository, message: &str) -> Result<(), MyError> {
    let signature = Signature::now("Diego", "hello@diegopisani.com")?;
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let head = repo.head()?.target().unwrap();
    let parent = repo.find_commit(head)?;

    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[&parent])?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let matches = App::new("rusteze-commit")
        .version("1.0")
        .author("Diego <hello@diegopisani.com>")
        .about("Generates git commit messages using OpenAI API")
        .arg(
            Arg::new("api_key")
                .short('k')
                .long("api-key")
                .value_name("API_KEY")
                .takes_value(true),
        )
        .get_matches();

    let api_key = matches.value_of("api_key").unwrap_or_else(|| {
        eprintln!("The --api-key option is required");
        std::process::exit(1);
    });

    let repo = Repository::open_from_env()?;
    let diff = get_diff(&repo)?;
    let message = get_commit_message(api_key, &diff).await?;
    create_commit(&repo, &message)?;

    Ok(())
}
