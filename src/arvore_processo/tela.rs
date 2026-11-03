//! Tela `procedimento_trabalhar`: a unidade publicada (`#lnkInfraUnidade`)
//! e o `src` assinado do único `iframe#ifrArvore`, conferido e resolvido.
use crate::instalacao::Instalacao;

/// URL absoluta da árvore, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDaArvore(String);

impl EnderecoDaArvore {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDaArvore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDaArvore(<redigido>)")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDoProcesso {
    pub(crate) unidade: String,
    pub(crate) arvore: EnderecoDaArvore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroTelaDoProcesso {
    HtmlDaTelaDoProcessoNaoInterpretado,
    IdentificacaoDaUnidadeAusenteNaTelaDoProcesso,
    UnidadeDaTelaDiferenteDaSolicitada,
    ArvoreAusenteNaTelaDoProcesso,
    ArvoresRepetidasNaTelaDoProcesso,
    EnderecoDaArvoreNaoReconhecido,
    EnderecoDaArvoreDeOutroProcesso,
}

impl ErroTelaDoProcesso {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDaTelaDoProcessoNaoInterpretado => "html_da_tela_do_processo_nao_interpretado",
            Self::IdentificacaoDaUnidadeAusenteNaTelaDoProcesso => {
                "identificacao_da_unidade_ausente_na_tela_do_processo"
            }
            Self::UnidadeDaTelaDiferenteDaSolicitada => "unidade_da_tela_diferente_da_solicitada",
            Self::ArvoreAusenteNaTelaDoProcesso => "arvore_ausente_na_tela_do_processo",
            Self::ArvoresRepetidasNaTelaDoProcesso => "arvores_repetidas_na_tela_do_processo",
            Self::EnderecoDaArvoreNaoReconhecido => "endereco_da_arvore_nao_reconhecido",
            Self::EnderecoDaArvoreDeOutroProcesso => "endereco_da_arvore_de_outro_processo",
        }
    }
}

/// Lê a tela servida em `url_da_tela` (absoluta, já conferida). `unidade`
/// pedida (`--unidade`) é comparada com espaços normalizados.
pub(crate) fn ler_tela_do_processo(
    instalacao: &Instalacao,
    url_da_tela: &str,
    html: &str,
    id: &str,
    unidade: Option<&str>,
) -> Result<TelaDoProcesso, ErroTelaDoProcesso> {
    let _ = (instalacao, url_da_tela, html, id, unidade);
    Err(ErroTelaDoProcesso::HtmlDaTelaDoProcessoNaoInterpretado)
}
