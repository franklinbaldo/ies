//! Leitura Web opt-in: link assinado privado; ainda não implementa GET.
use crate::{Cli, Domain, documento, documento_transporte};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroDocumentoRemoto {
    Numero(documento::NumeroDocumentoInvalido),
    QuantidadeDeDocumentos,
    OpcoesNaoComprovadas,
    Transporte(documento_transporte::ErroDocumentoTransporte),
    NumeroDoHtmlDiferenteDoSolicitado,
}
impl ErroDocumentoRemoto {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::Numero(e) => e.subcodigo(),
            Self::QuantidadeDeDocumentos => "get_documento_exige_um_numero",
            Self::OpcoesNaoComprovadas => "opcoes_incompativeis_com_documento_web",
            Self::Transporte(e) => e.subcodigo(),
            Self::NumeroDoHtmlDiferenteDoSolicitado => "numero_do_html_diferente_do_solicitado",
        }
    }
}
pub(super) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(comando) = &cli.command else { return None; };
    let _ = comando.url_documento_file.as_deref()?;
    Some(crate::erro_fonte(
        cli.json,
        "get_documento_ainda_nao_implementado",
        "consulta opt-in ainda não implementada",
    ))
}
