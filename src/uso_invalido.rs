//! Uso inválido que o clap não declara sozinho (combinações entre opções,
//! período): a mesma saída 2 do clap, antes de configuração ou rede. A
//! variante é a mensagem.
use clap::CommandFactory;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UsoInvalido {
    /// `--de` posterior a `--ate`.
    PeriodoInvertido,
    /// Opções de leitura de `ies processo`/`ies documento` junto com `buscar`.
    OpcoesDeLeituraComBuscar,
    /// O mesmo número SEI pedido mais de uma vez.
    NumeroSeiRepetido,
    /// `--html-privado` lê exatamente um documento.
    QuantidadeDeDocumentos,
    /// `--html-privado` não aceita opções que pediriam evidência do SEI.
    OpcoesDeDocumentoSemEvidencia,
}

pub(crate) fn recusar(uso: UsoInvalido) -> ! {
    crate::Cli::command()
        .error(clap::error::ErrorKind::ArgumentConflict, format!("{uso:?}"))
        .exit()
}
