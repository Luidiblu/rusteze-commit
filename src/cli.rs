use clap::{App, Arg};

pub fn get_cli_matches() -> ArgMatches {
    App::new("rusteze-commit")
        .version("0.1.0")
        .author("Autor 1 <autor1@email.com>, Autor 2 <autor2@email.com>")
        .about("Uma CLI que envia diferenças de arquivos após o git add para uma API e retorna uma mensagem de commit")
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
        .get_matches()
}
