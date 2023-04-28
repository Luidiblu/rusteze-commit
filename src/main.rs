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

#[derive(Debug)]
pub struct Config {
    pub use_emoji: bool,
    pub use_description: bool,
}

async fn get_commit_message(api_key: &str, prompt: &str, config: &Config) -> Result<Vec<String>, MyError> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "You are to act as the author of a commit message in git. Your mission is to create \
        clean and comprehensive commit messages in the conventional commit convention and \
        explain why a change was done. The diff is: {}\
        \nUse GitMoji convention to preface the commit: {}\
        \nAdd a short description of WHY the changes are done after the commit message: {}",
        prompt,
        config.use_emoji,
        config.use_description
    );

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

    let mut messages = Vec::new();
    if let Some(choices) = json["choices"].as_array() {
        for choice in choices {
            if let Some(message) = choice["message"]["content"].as_str() {
                messages.push(message.trim().to_string());
            }
        }
    } else if let Some(message) = json["choices"][0]["message"]["content"].as_str() {
        messages.push(message.trim().to_string());
    }

    if messages.is_empty() {
        Err(MyError::ApiError(format!("Failed to extract commit messages from API response: {:?}", json)))
    } else {
        Ok(messages)
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
        .arg(
            Arg::new("use_emoji")
                .short('e')
                .long("use-emoji")
                .takes_value(false)
        )
        .arg(
            Arg::new("use_description")
                .short('d')
                .long("use-description")
                .takes_value(false)
        )
        .get_matches();

    let api_key = matches.value_of("api_key").unwrap_or_else(|| {
        eprintln!("The --api-key option is required");
        std::process::exit(1);
    });

    let config = Config {
        use_emoji: matches.is_present("use_emoji"),
        use_description: matches.is_present("use_description"),
    };

    let repo = Repository::open_from_env()?;
    let diff = get_diff(&repo)?;
    let messages = get_commit_message(api_key, &diff, &config).await?;

    // Display the available choices and prompt the user to select one
    println!("Available commit messages:");
    for (index, message) in messages.iter().enumerate() {
        println!("{}. {}", index + 1, message);
    }
    println!("Please enter the number of the commit message you'd like to use:");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input)?;
    let choice_index: usize = user_input.trim().parse().unwrap_or(0);
    let message = &messages.get(choice_index - 1).ok_or(MyError::ApiError("Invalid choice".to_string()))?;

    create_commit(&repo, &message)?;

    Ok(())
}