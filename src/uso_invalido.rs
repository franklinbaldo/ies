//! Uso inválido que o clap não declara sozinho (combinações entre opções,
//! período): a mesma saída 2 do clap, antes de configuração ou rede.
use clap::CommandFactory;

pub(crate) fn recusar(mensagem: &str) -> ! {
    crate::Cli::command()
        .error(clap::error::ErrorKind::ArgumentConflict, mensagem)
        .exit()
}
