//! Ações de sobrestar e retomar publicadas na barra do nó PROCESSO (#269).
//! Só URLs servidas, com ID e `infra_hash` únicos; nunca executar JavaScript.
use super::barra_do_processo::ancoras_da_barra;
use super::concluir::{EnderecoDeEscrita, tela_do_no_da_raiz};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::menciona_acao;
use crate::instalacao::Instalacao;

const ONCLICK_RETOMAR: &str = "removerSobrestamentoProcesso();";

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcaoSobrestar {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    OutroProcesso,
    AcoesDeSobrestamentoRepetidasNaBarra,
    EnderecoDoSobrestamentoNaoReconhecido,
    TelaDoNoDaRetomadaNaoReconhecida,
}

/// Link para o formulário de sobrestamento: não é uma escrita.
pub(crate) fn ler_sobrestar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| menciona_acao(href, AcaoDoSei::ProcedimentoSobrestar))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [link] => {
            conferir_endereco_relativo(instalacao, link, AcaoDoSei::ProcedimentoSobrestar, id)
                .map(EnderecoDeEscrita::novo)
                .map(Some)
                .map_err(|erro| match erro {
                    EnderecoRecusado::NaoReconhecido => {
                        ErroAcaoSobrestar::EnderecoDoSobrestamentoNaoReconhecido
                    }
                    EnderecoRecusado::DeOutroProcesso => ErroAcaoSobrestar::OutroProcesso,
                })
        }
        _ => Err(ErroAcaoSobrestar::AcoesDeSobrestamentoRepetidasNaBarra),
    }
}

/// Retomar é uma função no HTML servido da raiz; sua URL só pode ser
/// procurada na tela do nó, depois de uma confirmação explícita.
pub(crate) fn ler_retomar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html)?;
    match ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some(ONCLICK_RETOMAR))
        .count()
    {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id)
            .map(Some)
            .ok_or(ErroAcaoSobrestar::TelaDoNoDaRetomadaNaoReconhecida),
        _ => Err(ErroAcaoSobrestar::AcoesDeSobrestamentoRepetidasNaBarra),
    }
}

/// A função de retirada pode estar no corpo inline (4.0) ou na variável
/// servida pela tela do nó (4.1/5.0); não se executa JavaScript.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEnderecoDeRetomar {
    FuncaoDeRetomarForaDaForma(#[source] super::concluir::FuncaoForaDaForma),
    VariavelDeRetomarForaDaForma(#[source] super::concluir::FuncaoForaDaForma),
    OutroProcesso,
    EnderecoDeRetomarAusenteNaTela,
    EnderecoDeRetomarNaoReconhecido,
    EnderecosDeRetomarRepetidosNaTela,
}

pub(crate) fn ler_endereco_de_retomar(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDeEscrita, ErroEnderecoDeRetomar> {
    let funcoes = super::concluir::urls_das_funcoes(html, "removerSobrestamentoProcesso")
        .map_err(ErroEnderecoDeRetomar::FuncaoDeRetomarForaDaForma)?;
    let variaveis = super::concluir::urls_das_variaveis(html, "linkRemoverSobrestamentoProcesso")
        .map_err(ErroEnderecoDeRetomar::VariavelDeRetomarForaDaForma)?;
    let url = match (funcoes.as_slice(), variaveis.as_slice()) {
        ([], []) => return Err(ErroEnderecoDeRetomar::EnderecoDeRetomarAusenteNaTela),
        ([url], []) | ([], [url]) => *url,
        _ => return Err(ErroEnderecoDeRetomar::EnderecosDeRetomarRepetidosNaTela),
    };
    conferir_endereco_relativo(
        instalacao,
        url,
        AcaoDoSei::ProcedimentoRemoverSobrestamento,
        id,
    )
    .map(EnderecoDeEscrita::novo)
    .map_err(|e| match e {
        EnderecoRecusado::NaoReconhecido => ErroEnderecoDeRetomar::EnderecoDeRetomarNaoReconhecido,
        EnderecoRecusado::DeOutroProcesso => ErroEnderecoDeRetomar::OutroProcesso,
    })
}
