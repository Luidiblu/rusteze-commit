mod cli;
mod git;
mod api;

use clap::ArgMatches;
use futures::executor::block_on;
use git2::Repository;
use tokio::runtime::Runtime;
use std::env;


fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        dotenv().ok();
        let api_key = env::var("OPENAI_API_KEY").expect("A variável de ambiente OPENAI_API_KEY não foi definida");
        let matches = cli::get_cli_matches();
        let repo_path = matches.value_of("path").unwrap();
        let api_url = "https://api.openai.com/v1/engines/davinci-codex/completions";

        match Repository::open(repo_path) {
            Ok(repo) => {
                let diff_after_add = git::get_diff_after_add(&repo).expect("Erro ao obter o diff após o git add");
                let commit_message = block_on(api::send_diff_to_api_and_get_commit_message(diff_after_add, api_url))
                    .expect("Erro ao obter a mensagem de commit da API");
                println!("Mensagem de commit sugerida: {}", commit_message);
            }
            Err(e) => {
                eprintln!("Erro ao abrir o repositório: {}", e);
                std::process::exit(1);
            }
        }
    });
}
