//! A barra (`Nos[i].acoes`) de um `DOCUMENTO` (Refs #88;
//! contracts/cli/documento-obter-versoes.md): o link assinado da lista de
//! versões (`documento_versao_listar`, "Versões do Documento") e a
//! declaração de edição (`onclick="editarConteudo('N');"` ou `'S'`).
//! Forma observada nas sete instalações do acervo público. Nenhum dos
//! links é montado (ADR 0003) e o editor nunca é aberto.
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use super::literais::{
    Literal, concatenacao_vazia_da_barra, indice_da_barra, ler_barra, ler_declaracao,
};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DAS_VERSOES: &str = "documento_versao_listar";

/// URL absoluta da lista de versões, assinada: `Debug` redigido.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDasVersoes(String);

impl EnderecoDasVersoes {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDasVersoes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDasVersoes(<redigido>)")
    }
}

/// O que a barra declara sobre a edição do conteúdo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdicaoNaBarra {
    /// `editarConteudo('N')`: editável, não assinado na montagem da árvore.
    NaoAssinado,
    /// `editarConteudo('S')`: editável, mas editar perde a assinatura.
    Assinado,
    /// Sem `editarConteudo`: a sessão não edita o conteúdo.
    NaoPublicada,
    /// `editarConteudo` com outro argumento, ou repetido: não observado.
    ForaDaForma,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BarraDoDocumento {
    pub(crate) versoes: Option<EnderecoDasVersoes>,
    pub(crate) edicao: EdicaoNaBarra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroBarraDoDocumento {
    BarraForaDaFormaObservada,
    BarrasRepetidasNaArvore,
    LinksDasVersoesRepetidosNaBarra,
    EnderecoDasVersoesNaoReconhecido,
    EnderecoDasVersoesDeOutroDocumento,
}

impl ErroBarraDoDocumento {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaFormaObservada => "barra_do_documento_fora_da_forma_observada",
            Self::BarrasRepetidasNaArvore => "barras_do_documento_repetidas_na_arvore",
            Self::LinksDasVersoesRepetidosNaBarra => "link_das_versoes_repetido_na_barra",
            Self::EnderecoDasVersoesNaoReconhecido => "link_das_versoes_nao_reconhecido",
            Self::EnderecoDasVersoesDeOutroDocumento => "link_das_versoes_de_outro_documento",
        }
    }
}

/// Lê a barra do `DOCUMENTO` de id `id_documento` numa árvore já conferida;
/// sem barra (ou sem o nó), nada publicado.
pub(crate) fn ler_barra_do_documento(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<BarraDoDocumento, ErroBarraDoDocumento> {
    // RED: shim sem leitura.
    let _ = (instalacao, html, id_documento, id_procedimento);
    Err(ErroBarraDoDocumento::BarraForaDaFormaObservada)
}

#[allow(dead_code)]
/// O primeiro `acao=` da consulta (a conferência recusa dois).
fn acao_do_href(href: &str) -> Option<&str> {
    let (_, consulta) = href.split_once('?')?;
    consulta
        .split('&')
        .find_map(|par| par.strip_prefix("acao="))
}

#[allow(dead_code)]
/// O texto da barra do nó `DOCUMENTO` de id `id`; duas barras são recusa.
fn barra_do_no(html: &str, id: &str) -> Result<Option<String>, ErroBarraDoDocumento> {
    use ErroBarraDoDocumento as E;
    let mut indice = None;
    for linha in html.split('\n') {
        if let Ok(Some((i, argumentos))) = ler_declaracao(linha)
            && let [Literal::Texto(tipo), Literal::Texto(a1), ..] = argumentos.as_slice()
            && tipo == "DOCUMENTO"
            && a1 == id
        {
            indice = Some(i);
            break;
        }
    }
    let Some(indice) = indice else {
        return Ok(None);
    };
    let mut texto = None;
    for linha in html.split('\n') {
        if indice_da_barra(linha) != Some(indice) || concatenacao_vazia_da_barra(linha).is_some() {
            continue;
        }
        let Some((_, lido)) = ler_barra(linha).map_err(|_| E::BarraForaDaFormaObservada)? else {
            return Err(E::BarraForaDaFormaObservada);
        };
        if texto.replace(lido).is_some() {
            return Err(E::BarrasRepetidasNaArvore);
        }
    }
    Ok(texto)
}
