//! Links de ciência no processo publicados pela árvore. A leitura da barra
//! usa o leitor compartilhado de atributos, e a URL da Lista de Ciências
//! vem exclusivamente da ação CIENCIAS da raiz. Nada executa JavaScript.
use super::barra_do_processo::{FalhaDaBarra, ancoras_da_barra};
use super::concluir::{EnderecoDeEscrita, tela_do_no_da_raiz};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, ler_chamada};
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCienciaNaArvore {
    BarraDeCienciaForaDaForma,
    CienciaRepetidaNaBarra,
    TelaDeCienciaNaoPublicada,
    ListaDeCienciasRepetidaNaArvore,
    ListaDeCienciasForaDaForma,
    EnderecoDaListaDeCienciasNaoReconhecido,
}

impl ErroCienciaNaArvore {
    pub(crate) const fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraDeCienciaForaDaForma => "barra_de_ciencia_fora_da_forma",
            Self::CienciaRepetidaNaBarra => "ciencia_repetida_na_barra",
            Self::TelaDeCienciaNaoPublicada => "tela_de_ciencia_nao_publicada",
            Self::ListaDeCienciasRepetidaNaArvore => "lista_de_ciencias_repetida_na_arvore",
            Self::ListaDeCienciasForaDaForma => "lista_de_ciencias_fora_da_forma",
            Self::EnderecoDaListaDeCienciasNaoReconhecido => "endereco_da_lista_de_ciencias_nao_reconhecido",
        }
    }
}

/// A barra só oferece ciência quando existe exatamente um botão da função.
/// A URL é a tela de nó, jamais a URL que já grava a ciência.
pub(crate) fn ciencia_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroCienciaNaArvore> {
    use ErroCienciaNaArvore as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora | FalhaDaBarra::Repetidas => E::BarraDeCienciaForaDaForma,
    })?;
    let total = ancoras.iter().filter(|a| a.onclick.as_deref() == Some("cienciaProcesso();")).count();
    match total {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id).map(Some).ok_or(E::TelaDeCienciaNaoPublicada),
        _ => Err(E::CienciaRepetidaNaBarra),
    }
}

/// Link assinado único da Lista de Ciências do PROCESSO na árvore novamente
/// aberta. A do documento (código CD<id>) não serve como prova do processo.
pub(crate) fn lista_do_processo(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroCienciaNaArvore> {
    use ErroCienciaNaArvore as E;
    let mut urls = Vec::new();
    for linha in html.lines() {
        if !linha.contains("new infraArvoreAcao(") || !linha.contains("CIENCIAS") {
            continue;
        }
        let argumentos = ler_chamada(linha, "NosAcoes[", "infraArvoreAcao")
            .ok_or(E::ListaDeCienciasForaDaForma)?;
        match argumentos.as_slice() {
            [Literal::Texto(tipo), Literal::Texto(codigo), Literal::Texto(alvo), Literal::Texto(url), ..]
                if tipo == "CIENCIAS" && codigo == "C" && alvo == id => urls.push(url.clone()),
            [Literal::Texto(tipo), ..] if tipo == "CIENCIAS" => {
                // A ação CD... é do documento e não confere ciência no processo.
            }
            _ => return Err(E::ListaDeCienciasForaDaForma),
        }
    }
    let url = match urls.as_slice() {
        [] => return Ok(None),
        [url] => url,
        _ => return Err(E::ListaDeCienciasRepetidaNaArvore),
    };
    conferir_endereco_relativo(instalacao, url, "protocolo_ciencia_listar", id)
        .map(|url| Some(EnderecoDeEscrita::novo(url)))
        .map_err(|_: EnderecoRecusado| E::EnderecoDaListaDeCienciasNaoReconhecido)
}
