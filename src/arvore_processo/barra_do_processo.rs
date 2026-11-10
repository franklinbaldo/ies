//! Os links das propriedades e dos marcadores na barra do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-propriedades.md). Forma observada na `UnB`
//! 4.0.9 (03 e 04/11/2026): a linha `Nos[i].acoes = '<literal>';` do nó
//! raiz, com `<a href="controlador.php?acao=procedimento_alterar&…">` (ou
//! `procedimento_consultar`, com o processo aberto só noutra unidade). As
//! tags são lidas pelo tokenizador de atributos; o ícone e o `title` não
//! classificam nada.
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, indice_da_barra, ler_barra, ler_declaracao};
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;
use crate::marcadores_processo::{self, EnderecoDosMarcadores, ErroMarcadores};
use crate::processo_comentarios::{EnderecoDosComentarios, ErroLinkDosComentarios};
use crate::propriedades_processo::{
    EnderecoDasPropriedades, ErroPropriedades, FonteDasPropriedades,
};

/// O link da tela de propriedades publicado na barra do processo `id`;
/// `Ok(None)` se a barra não publica `procedimento_alterar` nem
/// `procedimento_consultar`. Os dois juntos: vale o de alterar.
pub(crate) fn ler_link_das_propriedades(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDasPropriedades>, ErroPropriedades> {
    let falha = |f: FalhaDaBarra| match f {
        FalhaDaBarra::Fora => ErroPropriedades::BarraDoProcessoForaDaFormaObservada,
        FalhaDaBarra::Repetidas => ErroPropriedades::BarrasDoProcessoRepetidasNaArvore,
    };
    let (mut alterar, mut consultar) = (Vec::new(), Vec::new());
    for href in hrefs_da_barra(html).map_err(falha)? {
        match acao_do_href(&href) {
            Some(acao) if acao == FonteDasPropriedades::Alterar.acao() => alterar.push(href),
            Some(acao) if acao == FonteDasPropriedades::Consultar.acao() => consultar.push(href),
            _ => {}
        }
    }
    let (fonte, achados) = if alterar.is_empty() {
        (FonteDasPropriedades::Consultar, consultar)
    } else {
        (FonteDasPropriedades::Alterar, alterar)
    };
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(ErroPropriedades::LinksDasPropriedadesRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, fonte.acao(), id)
        .map(|url| Some(EnderecoDasPropriedades::novo(fonte, url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                ErroPropriedades::EnderecoDasPropriedadesNaoReconhecido
            }
            EnderecoRecusado::DeOutroProcesso => {
                ErroPropriedades::EnderecoDasPropriedadesDeOutroProcesso
            }
        })
}

/// Anotações na origem: somente o link servido no nó PROCESSO.
/// O GET deste endereço abre o formulário, não submete anotação.
pub(crate) fn ler_link_das_anotacoes(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<String>, ErroHtml> {
    let hrefs =
        hrefs_da_barra(html).map_err(|_| ErroHtml::html("barra_de_anotacoes_nao_interpretada"))?;
    let candidatos: Vec<_> = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some("anotacao_registrar"))
        .collect();
    match candidatos.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, "anotacao_registrar", id)
            .map(Some)
            .map_err(|_| ErroHtml::valor("endereco_de_anotacoes_nao_reconhecido")),
        _ => Err(ErroHtml::repetido("links_de_anotacoes_repetidos")),
    }
}

/// Recusas do link de controle de prazo; URLs assinadas nunca aparecem no erro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkPrazo {
    BarraNaoInterpretada,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroProcesso,
}

/// Ação de prazo servida na barra do próprio processo, nunca fabricada.
pub(crate) fn ler_link_do_prazo(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<String>, ErroLinkPrazo> {
    let hrefs = hrefs_da_barra(html).map_err(|_| ErroLinkPrazo::BarraNaoInterpretada)?;
    let candidatos: Vec<_> = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some("controle_prazo_definir"))
        .collect();
    let href = match candidatos.as_slice() {
        [] => return Ok(None),
        [unico] => *unico,
        _ => return Err(ErroLinkPrazo::LinksRepetidos),
    };
    conferir_endereco_relativo(instalacao, href, "controle_prazo_definir", id)
        .map(Some)
        .map_err(|falha| match falha {
            EnderecoRecusado::NaoReconhecido => ErroLinkPrazo::EnderecoNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => ErroLinkPrazo::EnderecoDeOutroProcesso,
        })
}

/// O link da tela "Marcadores do Processo" (`andamento_marcador_gerenciar`)
/// publicado na barra do processo `id`; `Ok(None)` se a barra não o publica
/// (contracts/cli/processo-obter-marcadores.md).
pub(crate) fn ler_link_dos_marcadores(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDosMarcadores>, ErroMarcadores> {
    let hrefs = hrefs_da_barra(html).map_err(|f| match f {
        FalhaDaBarra::Fora => ErroMarcadores::BarraDoProcessoForaDaFormaObservada,
        FalhaDaBarra::Repetidas => ErroMarcadores::BarrasDoProcessoRepetidasNaArvore,
    })?;
    let achados: Vec<_> = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some(marcadores_processo::ACAO))
        .collect();
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(ErroMarcadores::LinksDosMarcadoresRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, marcadores_processo::ACAO, id)
        .map(|url| Some(EnderecoDosMarcadores::novo(url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => ErroMarcadores::EnderecoDosMarcadoresNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => {
                ErroMarcadores::EnderecoDosMarcadoresDeOutroProcesso
            }
        })
}

/// Link assinado de `comentario_listar` da própria barra do processo.
/// Não forma URL, não executa JS e não segue links de documentos.
pub(crate) fn ler_link_dos_comentarios(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDosComentarios>, ErroLinkDosComentarios> {
    let hrefs = hrefs_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => ErroLinkDosComentarios::Html(ErroHtml::valor(
            "barra_dos_comentarios_fora_da_forma_observada",
        )),
        FalhaDaBarra::Repetidas => ErroLinkDosComentarios::Html(ErroHtml::repetido(
            "barras_dos_comentarios_repetidas_na_arvore",
        )),
    })?;
    let achados: Vec<_> = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some("comentario_listar"))
        .collect();
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [unico] => unico,
        _ => return Err(ErroHtml::repetido("links_dos_comentarios_repetidos_na_barra").into()),
    };
    conferir_endereco_relativo(instalacao, href, "comentario_listar", id)
        .map(|url| Some(EnderecoDosComentarios::novo(url)))
        .map_err(|falha| match falha {
            EnderecoRecusado::NaoReconhecido => ErroLinkDosComentarios::Html(ErroHtml::valor(
                "endereco_dos_comentarios_nao_reconhecido",
            )),
            EnderecoRecusado::DeOutroProcesso => ErroLinkDosComentarios::EnderecoDeOutroProcesso,
        })
}

/// Barra fora da forma observada, ou duas barras para o nó `PROCESSO`.
pub(super) enum FalhaDaBarra {
    Fora,
    Repetidas,
}

/// Os `href` (entidades decodificadas) das tags `<a>` da barra do nó
/// `PROCESSO`, na ordem servida; sem barra, nenhum.
fn hrefs_da_barra(html: &str) -> Result<Vec<String>, FalhaDaBarra> {
    Ok(ancoras_da_barra(html)?
        .into_iter()
        .filter_map(|a| a.href)
        .collect())
}

/// Uma `<a>` da barra: `href` e `onclick`, entidades decodificadas.
pub(super) struct AncoraDaBarra {
    pub(super) href: Option<String>,
    pub(super) onclick: Option<String>,
}

/// As tags `<a>` da barra do nó `PROCESSO`, na ordem servida; sem barra,
/// nenhuma.
pub(super) fn ancoras_da_barra(html: &str) -> Result<Vec<AncoraDaBarra>, FalhaDaBarra> {
    let Some(texto) = barra_da_raiz(html)? else {
        return Ok(Vec::new());
    };
    let dom = tl::parse(&texto, tl::ParserOptions::default()).map_err(|_| FalhaDaBarra::Fora)?;
    let parser = dom.parser();
    let mut ancoras = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(|_| FalhaDaBarra::Fora)?;
        let decodificado = |nome: &'static str| -> Result<Option<String>, FalhaDaBarra> {
            match lidos.valor(nome).map_err(|_| FalhaDaBarra::Fora)? {
                None => Ok(None),
                Some(bruto) => decodificar_atributo(bruto)
                    .map(Some)
                    .ok_or(FalhaDaBarra::Fora),
            }
        };
        ancoras.push(AncoraDaBarra {
            href: decodificado("href")?,
            onclick: decodificado("onclick")?,
        });
    }
    Ok(ancoras)
}
/// O primeiro `acao=` da consulta do `href` (a conferência recusa dois).
pub(super) fn acao_do_href(href: &str) -> Option<&str> {
    let (_, consulta) = href.split_once('?')?;
    consulta
        .split('&')
        .find_map(|par| par.strip_prefix("acao="))
}

/// O texto da barra do nó `PROCESSO`; duas barras para ele são recusa.
fn barra_da_raiz(html: &str) -> Result<Option<String>, FalhaDaBarra> {
    let mut raiz = None;
    for linha in html.split('\n') {
        if let Some((i, argumentos)) = ler_declaracao(linha).map_err(|_| FalhaDaBarra::Fora)?
            && argumentos.first() == Some(&Literal::Texto("PROCESSO".into()))
        {
            raiz = Some(i);
            break;
        }
    }
    let Some(raiz) = raiz else {
        return Ok(None);
    };
    let mut texto = None;
    for linha in html.split('\n') {
        if indice_da_barra(linha) != Some(raiz) {
            continue;
        }
        let Some((_, lido)) = ler_barra(linha).map_err(|_| FalhaDaBarra::Fora)? else {
            return Err(FalhaDaBarra::Fora);
        };
        if texto.is_some() {
            return Err(FalhaDaBarra::Repetidas);
        }
        texto = Some(lido);
    }
    Ok(texto)
}

/// URL assinada da escolha de tipos de documento, nunca exposta por Debug.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDoIncluirDocumento(String);
impl EnderecoDoIncluirDocumento {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for EnderecoDoIncluirDocumento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDoIncluirDocumento(<redigido>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkIncluirDocumento {
    BarraForaDaForma,
    BarraRepetida,
    LinkRepetido,
    EnderecoNaoReconhecido,
    EnderecoDeOutroProcesso,
}

/// Extrai exclusivamente o link publicado na barra do nó PROCESSO.
/// Nunca constrói URLs assinadas nem executa JavaScript.
pub(crate) fn ler_link_de_incluir_documento(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDoIncluirDocumento>, ErroLinkIncluirDocumento> {
    let hrefs = hrefs_da_barra(html).map_err(|erro| match erro {
        FalhaDaBarra::Fora => ErroLinkIncluirDocumento::BarraForaDaForma,
        FalhaDaBarra::Repetidas => ErroLinkIncluirDocumento::BarraRepetida,
    })?;
    let mut links = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some("documento_escolher_tipo"));
    let Some(href) = links.next() else {
        return Ok(None);
    };
    if links.next().is_some() {
        return Err(ErroLinkIncluirDocumento::LinkRepetido);
    }
    conferir_endereco_relativo(instalacao, href, "documento_escolher_tipo", id)
        .map(|url| Some(EnderecoDoIncluirDocumento(url)))
        .map_err(|erro| match erro {
            EnderecoRecusado::NaoReconhecido => ErroLinkIncluirDocumento::EnderecoNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => ErroLinkIncluirDocumento::EnderecoDeOutroProcesso,
        })
}
