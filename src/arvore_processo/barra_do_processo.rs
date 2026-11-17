//! Os links das propriedades e dos marcadores na barra do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-propriedades.md). Forma observada na `UnB`
//! 4.0.9 (03 e 04/11/2026): a linha `Nos[i].acoes = '<literal>';` do nó
//! raiz, com `<a href="controlador.php?acao=procedimento_alterar&…">` (ou
//! `procedimento_consultar`, com o processo aberto só noutra unidade). As
//! tags são lidas pelo tokenizador de atributos; o ícone e o `title` não
//! classificam nada.
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, indice_da_barra, ler_barra, ler_declaracao};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::menciona_acao;
use crate::instalacao::Instalacao;
use crate::marcadores_processo::{EnderecoDosMarcadores, ErroMarcadores};
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
    let (mut alterar, mut consultar) = (Vec::new(), Vec::new());
    for href in hrefs_da_barra(html)? {
        if menciona_acao(&href, FonteDasPropriedades::Alterar.acao()) {
            alterar.push(href);
        } else if menciona_acao(&href, FonteDasPropriedades::Consultar.acao()) {
            consultar.push(href);
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

/// Recusas da leitura do link de anotações na barra do processo.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDasAnotacoes {
    BarraDeAnotacoesIlegivel(#[source] super::ErroBarraDoProcesso),
    EnderecoDeAnotacoesRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    LinksDeAnotacoesRepetidos,
}

/// Anotações na origem: somente o link servido no nó PROCESSO.
/// O GET deste endereço abre o formulário, não submete anotação.
pub(crate) fn ler_link_das_anotacoes(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<String>, ErroLinkDasAnotacoes> {
    let hrefs = hrefs_da_barra(html).map_err(ErroLinkDasAnotacoes::BarraDeAnotacoesIlegivel)?;
    let candidatos: Vec<_> = hrefs
        .iter()
        .filter(|href| menciona_acao(href, AcaoDoSei::AnotacaoRegistrar))
        .collect();
    match candidatos.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, AcaoDoSei::AnotacaoRegistrar, id)
            .map(Some)
            .map_err(ErroLinkDasAnotacoes::EnderecoDeAnotacoesRecusado),
        _ => Err(ErroLinkDasAnotacoes::LinksDeAnotacoesRepetidos),
    }
}

/// Recusas do link de controle de prazo; URLs assinadas nunca aparecem no erro.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkPrazo {
    BarraDoPrazoIlegivel(#[source] super::ErroBarraDoProcesso),
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
    let hrefs = hrefs_da_barra(html).map_err(ErroLinkPrazo::BarraDoPrazoIlegivel)?;
    let candidatos: Vec<_> = hrefs
        .iter()
        .filter(|href| menciona_acao(href, AcaoDoSei::ControlePrazoDefinir))
        .collect();
    let href = match candidatos.as_slice() {
        [] => return Ok(None),
        [unico] => *unico,
        _ => return Err(ErroLinkPrazo::LinksRepetidos),
    };
    conferir_endereco_relativo(instalacao, href, AcaoDoSei::ControlePrazoDefinir, id)
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
    let hrefs = hrefs_da_barra(html)?;
    let achados: Vec<_> = hrefs
        .iter()
        .filter(|href| menciona_acao(href, AcaoDoSei::AndamentoMarcadorGerenciar))
        .collect();
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(ErroMarcadores::LinksDosMarcadoresRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, AcaoDoSei::AndamentoMarcadorGerenciar, id)
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
    let hrefs = hrefs_da_barra(html)?;
    let achados: Vec<_> = hrefs
        .iter()
        .filter(|href| menciona_acao(href, AcaoDoSei::ComentarioListar))
        .collect();
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [unico] => unico,
        _ => return Err(ErroLinkDosComentarios::LinksDosComentariosRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, AcaoDoSei::ComentarioListar, id)
        .map(|url| Some(EnderecoDosComentarios::novo(url)))
        .map_err(|falha| match falha {
            EnderecoRecusado::NaoReconhecido => {
                ErroLinkDosComentarios::EnderecoDosComentariosNaoReconhecido
            }
            EnderecoRecusado::DeOutroProcesso => ErroLinkDosComentarios::EnderecoDeOutroProcesso,
        })
}

/// A barra do nó `PROCESSO` (`Nos[0].acoes`) como o leitor a encontrou.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBarraDoProcesso {
    DeclaracaoDaBarraIlegivel(#[source] super::ErroArvore),
    LinhaDaBarraIlegivel(#[source] super::ErroArvore),
    BarraDoProcessoForaDaFormaObservada,
    BarrasDoProcessoRepetidasNaArvore,
}

/// Os `href` (entidades decodificadas) das tags `<a>` da barra do nó
/// `PROCESSO`, na ordem servida; sem barra, nenhum.
fn hrefs_da_barra(html: &str) -> Result<Vec<String>, ErroBarraDoProcesso> {
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
pub(super) fn ancoras_da_barra(html: &str) -> Result<Vec<AncoraDaBarra>, ErroBarraDoProcesso> {
    let Some(texto) = barra_da_raiz(html)? else {
        return Ok(Vec::new());
    };
    let dom = crate::tela_html::arvore(&texto);
    let mut ancoras = Vec::new();
    for no in dom.elementos("a") {
        ancoras.push(AncoraDaBarra {
            href: no.atributo("href").map(str::to_owned),
            onclick: no.atributo("onclick").map(str::to_owned),
        });
    }
    Ok(ancoras)
}

/// O texto da barra do nó `PROCESSO`; duas barras para ele são recusa.
fn barra_da_raiz(html: &str) -> Result<Option<crate::tela_html::HtmlServido>, ErroBarraDoProcesso> {
    let mut raiz = None;
    for linha in html.split('\n') {
        if let Some((i, argumentos)) =
            ler_declaracao(linha).map_err(ErroBarraDoProcesso::DeclaracaoDaBarraIlegivel)?
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
        let Some((_, lido)) =
            ler_barra(linha).map_err(ErroBarraDoProcesso::LinhaDaBarraIlegivel)?
        else {
            return Err(ErroBarraDoProcesso::BarraDoProcessoForaDaFormaObservada);
        };
        if texto.is_some() {
            return Err(ErroBarraDoProcesso::BarrasDoProcessoRepetidasNaArvore);
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkIncluirDocumento {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
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
    let hrefs = hrefs_da_barra(html)?;
    let mut links = hrefs
        .iter()
        .filter(|href| menciona_acao(href, AcaoDoSei::DocumentoEscolherTipo));
    let Some(href) = links.next() else {
        return Ok(None);
    };
    if links.next().is_some() {
        return Err(ErroLinkIncluirDocumento::LinkRepetido);
    }
    conferir_endereco_relativo(instalacao, href, AcaoDoSei::DocumentoEscolherTipo, id)
        .map(|url| Some(EnderecoDoIncluirDocumento(url)))
        .map_err(|erro| match erro {
            EnderecoRecusado::NaoReconhecido => ErroLinkIncluirDocumento::EnderecoNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => ErroLinkIncluirDocumento::EnderecoDeOutroProcesso,
        })
}
