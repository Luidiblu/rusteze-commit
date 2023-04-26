use clap::{App, Arg};
use git2::{DiffOptions, Repository, StatusOptions};
use ansi_term::Colour::{Green, Red, Yellow};

fn main() {
    let matches = App::new("rusteze-commit")
        .version("0.1.0")
        .author("Diego <hello@diegopisani.com>, Prey <felipeprey@hotmail.com>")
        .about("Commita arquivos alterados no repositório Git")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .about("Caminho do diretório do repositório Git")
                .required(false)
                .default_value(".")
                .takes_value(true),
        )
        .get_matches();

    let repo_path = matches.value_of("path").unwrap();

    match Repository::open(repo_path) {
        Ok(repo) => {
            print_status(&repo);
            print_diff(&repo);
        }
        Err(e) => {
            eprintln!("Erro ao abrir o repositório: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_status(repo: &Repository) {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let statuses = match repo.statuses(Some(&mut status_opts)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao listar o status: {}", e);
            std::process::exit(1);
        }
    };

    println!("{}:", Yellow.paint("Arquivos adicionados"));
    for entry in statuses.iter().filter(|e| e.status().is_index_new()) {
        let file_path = entry.path().unwrap().to_string_lossy();
        println!("{}", Green.paint(file_path));
    }
}

fn print_diff(repo: &Repository) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Erro ao obter o HEAD: {}", e);
            std::process::exit(1);
        }
    };

    let head_commit = match head.peel_to_commit() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Erro ao obter o commit HEAD: {}", e);
            std::process::exit(1);
        }
    };

    let head_tree = match head_commit.tree() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erro ao obter a árvore HEAD: {}", e);
            std::process::exit(1);
        }
    };

    let index = match repo.index() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Erro ao obter o índice: {}", e);
            std::process::exit(1);
        }
    };

    let index_tree = match index.into_tree() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erro ao converter o índice em árvore: {}", e);
            std::process::exit(1);
        }
    };

    let diff = match repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), None) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Erro ao calcular o diff: {}", e);
            std::process::exit(1);
        }
    };

    let diff_stats = match diff.stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erro ao obter as estatísticas do diff: {}", e);
            std::process::exit(1);
        }
    };

    let formatted_diff_stats = match diff_stats.to_buf(git2::DiffStatsFormat::FULL, 80) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("Erro ao formatar as estatísticas do diff: {}", e);
            std::process::exit(1);
        }
    };

    let diff_output = String::from_utf8_lossy(&formatted_diff_stats).to_string();
    println!("\n{}:", Red.paint("Diferenças"));
    println!("{}", diff_output);
}
