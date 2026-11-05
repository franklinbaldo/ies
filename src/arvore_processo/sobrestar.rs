//! Ações de sobrestar e retomar publicadas na barra do nó PROCESSO (#269).
//! Só URLs servidas, com ID e `infra_hash` únicos; nunca executar JavaScript.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::{EnderecoDeEscrita, tela_do_no_da_raiz};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

const ACAO_SOBRESTAR: &str = "procedimento_sobrestar";
const ONCLICK_RETOMAR: &str = "removerSobrestamentoProcesso();";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAcaoSobrestar {
    BarraInvalida,
    AcoesRepetidas,
    EnderecoNaoReconhecido,
    OutroProcesso,
    TelaDoNoNaoReconhecida,
}

impl ErroAcaoSobrestar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraInvalida => "barra_do_sobrestamento_nao_interpretada",
            Self::AcoesRepetidas => "acoes_de_sobrestamento_repetidas_na_barra",
            Self::EnderecoNaoReconhecido => "endereco_do_sobrestamento_nao_reconhecido",
            Self::OutroProcesso => "endereco_do_sobrestamento_de_outro_processo",
            Self::TelaDoNoNaoReconhecida => "tela_do_no_da_retomada_nao_reconhecida",
        }
    }
}

fn mapear_barra(_: FalhaDaBarra) -> ErroAcaoSobrestar {
    ErroAcaoSobrestar::BarraInvalida
}

/// Link para o formulário de sobrestamento: não é uma escrita.
pub(crate) fn ler_sobrestar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html).map_err(mapear_barra)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_SOBRESTAR))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [link] => conferir_endereco_relativo(instalacao, link, ACAO_SOBRESTAR, id)
            .map(EnderecoDeEscrita::novo)
            .map(Some)
            .map_err(|erro| match erro {
                EnderecoRecusado::NaoReconhecido => ErroAcaoSobrestar::EnderecoNaoReconhecido,
                EnderecoRecusado::DeOutroProcesso => ErroAcaoSobrestar::OutroProcesso,
            }),
        _ => Err(ErroAcaoSobrestar::AcoesRepetidas),
    }
}

/// Retomar é uma função no HTML servido da raiz; sua URL só pode ser
/// procurada na tela do nó, depois de uma confirmação explícita.
pub(crate) fn ler_retomar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html).map_err(mapear_barra)?;
    match ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some(ONCLICK_RETOMAR))
        .count()
    {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id)
            .map(Some)
            .ok_or(ErroAcaoSobrestar::TelaDoNoNaoReconhecida),
        _ => Err(ErroAcaoSobrestar::AcoesRepetidas),
    }
}

/// A função de retirada pode estar no corpo inline (4.0) ou na variável
/// servida pela tela do nó (4.1/5.0); não se executa JavaScript.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroEnderecoDeRetomar {
    Ausente,
    ForaDaForma,
    Repetido,
    NaoReconhecido,
    OutroProcesso,
}
impl ErroEnderecoDeRetomar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Ausente => "endereco_de_retomar_ausente_na_tela",
            Self::ForaDaForma => "endereco_de_retomar_fora_da_forma",
            Self::Repetido => "enderecos_de_retomar_repetidos_na_tela",
            Self::NaoReconhecido => "endereco_de_retomar_nao_reconhecido",
            Self::OutroProcesso => "endereco_de_retomar_de_outro_processo",
        }
    }
}

pub(crate) fn ler_endereco_de_retomar(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDeEscrita, ErroEnderecoDeRetomar> {
    use ErroEnderecoDeRetomar as E;
    let funcoes = super::concluir::urls_das_funcoes(html, "removerSobrestamentoProcesso")
        .map_err(|_| E::ForaDaForma)?;
    let mut variaveis = Vec::new();
    for linha in html.lines().map(str::trim) {
        if !linha.starts_with("var linkRemoverSobrestamentoProcesso") {
            continue;
        }
        let url = linha
            .strip_prefix("var linkRemoverSobrestamentoProcesso = '")
            .and_then(|s| s.strip_suffix("';"))
            .ok_or(E::ForaDaForma)?;
        variaveis.push(url);
    }
    let url = match (funcoes.as_slice(), variaveis.as_slice()) {
        ([], []) => return Err(E::Ausente),
        ([url], []) | ([], [url]) => *url,
        _ => return Err(E::Repetido),
    };
    conferir_endereco_relativo(instalacao, url, "procedimento_remover_sobrestamento", id)
        .map(EnderecoDeEscrita::novo)
        .map_err(|e| match e {
            EnderecoRecusado::NaoReconhecido => E::NaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => E::OutroProcesso,
        })
}
