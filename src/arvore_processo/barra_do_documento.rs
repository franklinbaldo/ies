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
use crate::erro_html::ErroHtml;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBarraDoDocumento {
    BarrasRepetidasNaArvore,
    EnderecoDasVersoesDeOutroDocumento,
    Html(#[from] ErroHtml),
}

/// Lê a barra do `DOCUMENTO` de id `id_documento` numa árvore já conferida;
/// sem barra (ou sem o nó), nada publicado.
pub(crate) fn ler_barra_do_documento(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<BarraDoDocumento, ErroBarraDoDocumento> {
    let nada = BarraDoDocumento {
        versoes: None,
        edicao: EdicaoNaBarra::NaoPublicada,
    };
    let Some(texto) = barra_do_no(html, id_documento)? else {
        return Ok(nada);
    };
    let dom = tl::parse(&texto, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::valor("barra_do_documento_fora_da_forma_observada"))?;
    let parser = dom.parser();
    let (mut links, mut edicoes) = (Vec::new(), Vec::new());
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag)
            .map_err(|e| ErroHtml::atributo("barra_do_documento_fora_da_forma_observada", e))?;
        let valor = |nome| -> Result<Option<String>, ErroBarraDoDocumento> {
            match lidos.valor(nome).map_err(|e| {
                ErroBarraDoDocumento::Html(ErroHtml::atributo(
                    "barra_do_documento_fora_da_forma_observada",
                    e,
                ))
            })? {
                Some(bruto) => {
                    decodificar_atributo(bruto)
                        .map(Some)
                        .ok_or(ErroBarraDoDocumento::Html(ErroHtml::valor(
                            "barra_do_documento_fora_da_forma_observada",
                        )))
                }
                None => Ok(None),
            }
        };
        if let Some(href) = valor("href")?
            && acao_do_href(&href) == Some(ACAO_DAS_VERSOES)
        {
            links.push(href);
        }
        if let Some(onclick) = valor("onclick")?
            && onclick.contains("editarConteudo")
        {
            edicoes.push(match onclick.as_str() {
                "editarConteudo('N');" => EdicaoNaBarra::NaoAssinado,
                "editarConteudo('S');" => EdicaoNaBarra::Assinado,
                _ => EdicaoNaBarra::ForaDaForma,
            });
        }
    }
    let edicao = match edicoes.as_slice() {
        [] => EdicaoNaBarra::NaoPublicada,
        [uma] => *uma,
        _ => EdicaoNaBarra::ForaDaForma,
    };
    let href = match links.as_slice() {
        [] => return Ok(BarraDoDocumento { edicao, ..nada }),
        [href] => href,
        _ => return Err(ErroHtml::repetido("link_das_versoes_repetido_na_barra").into()),
    };
    let recusa = |r| match r {
        EnderecoRecusado::NaoReconhecido => {
            ErroBarraDoDocumento::Html(ErroHtml::valor("link_das_versoes_nao_reconhecido"))
        }
        EnderecoRecusado::DeOutroProcesso => {
            ErroBarraDoDocumento::EnderecoDasVersoesDeOutroDocumento
        }
    };
    conferir_endereco_relativo_por(
        instalacao,
        href,
        ACAO_DAS_VERSOES,
        "id_procedimento",
        id_procedimento,
    )
    .map_err(recusa)?;
    let url = conferir_endereco_relativo_por(
        instalacao,
        href,
        ACAO_DAS_VERSOES,
        "id_documento",
        id_documento,
    )
    .map_err(recusa)?;
    Ok(BarraDoDocumento {
        versoes: Some(EnderecoDasVersoes(url)),
        edicao,
    })
}

/// O primeiro `acao=` da consulta (a conferência recusa dois).
fn acao_do_href(href: &str) -> Option<&str> {
    let (_, consulta) = href.split_once('?')?;
    consulta
        .split('&')
        .find_map(|par| par.strip_prefix("acao="))
}

/// O texto da barra do nó `DOCUMENTO` de id `id`; duas barras são recusa.
pub(crate) fn barra_do_no(html: &str, id: &str) -> Result<Option<String>, ErroBarraDoDocumento> {
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
        let Some((_, lido)) = ler_barra(linha)
            .map_err(|_| ErroHtml::valor("barra_do_documento_fora_da_forma_observada"))?
        else {
            return Err(ErroHtml::valor("barra_do_documento_fora_da_forma_observada").into());
        };
        if texto.replace(lido).is_some() {
            return Err(ErroBarraDoDocumento::BarrasRepetidasNaArvore);
        }
    }
    Ok(texto)
}
