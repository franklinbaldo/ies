//! Leitores puros de blocos do SEI: tabela por cabeçalho, sem JS nem escrita.
//! Oráculo de cenários: Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador, menciona_acao,
};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TipoBloco {
    Assinatura,
    Interno,
    Reuniao,
}
impl TipoBloco {
    pub(crate) const fn acao(self) -> AcaoDoSei {
        match self {
            Self::Assinatura => AcaoDoSei::BlocoAssinaturaListar,
            Self::Interno => AcaoDoSei::BlocoInternoListar,
            Self::Reuniao => AcaoDoSei::BlocoReuniaoListar,
        }
    }
    pub(crate) const fn rotulo(self) -> &'static str {
        match self {
            Self::Assinatura => "assinatura",
            Self::Interno => "interno",
            Self::Reuniao => "reuniao",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBloco {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    FaixaDeBlocosIncoerente,
    PaginaDeBlocosNaoImplementada,
    BlocoNaoEncontradoNaPagina,
    CabecalhoDeBlocosNaoReconhecido,
    TotalDaLegendaIlegivel(#[source] std::num::ParseIntError),
    InicioDaFaixaIlegivel(#[source] std::num::ParseIntError),
    FimDaFaixaIlegivel(#[source] std::num::ParseIntError),
    /// A consulta do link de bloco não se deixa ler em pares.
    ConsultaDoLinkDeBloco(#[source] ErroConsultaDoControlador),
    /// `acao`, `infra_hash` do link de bloco ausentes ou repetidos.
    ChaveDoLinkDeBloco(#[source] ErroChaveDaConsulta),
    LegendaDeBlocosInvalida,
    LinhaDeBlocoForaDaForma,
    LinkDeBlocosAusente,
    LinkDeBlocosNaoReconhecido,
    LinksDeBlocosRepetidos,
    NumeroDoBlocoInvalido,
    NumeroDoBlocoRepetido,
    TabelaDeBlocosAusente,
    TabelasDeBlocosRepetidas,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BlocoLido {
    pub(crate) numero: String,
    pub(crate) tipo: TipoBloco,
    pub(crate) estado: String,
    pub(crate) descricao: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Faixa {
    pub(crate) inicio: usize,
    pub(crate) fim: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ListaDeBlocos {
    pub(crate) tipo: TipoBloco,
    pub(crate) total_declarado: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) faixa_observada: Option<Faixa>,
    pub(crate) itens: Vec<BlocoLido>,
}

/// A lista pública e os hrefs mantidos exclusivamente em memória (NUNCA
/// serializados, impressos, incluídos em Debug ou fixtures).
pub(crate) struct ListaComLinks {
    pub(crate) lista: ListaDeBlocos,
    links: Vec<String>,
}
impl ListaComLinks {
    pub(crate) fn href_do_bloco(&self, numero: &str) -> Result<&str, ErroBloco> {
        let mut encontrados = self
            .lista
            .itens
            .iter()
            .zip(&self.links)
            .filter(|(b, _)| b.numero == numero);
        match (encontrados.next(), encontrados.next()) {
            (Some((_, href)), None) => Ok(href),
            _ => Err(ErroBloco::BlocoNaoEncontradoNaPagina),
        }
    }
}

fn texto_no(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&no.inner_text(parser))
        .into_owned()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn texto_tag(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&tag.inner_text(parser))
        .into_owned()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cabecalhos(tabela: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Result<Vec<String>, ErroBloco> {
    let linhas = tabela
        .query_selector(parser, "tr")
        .ok_or(ErroBloco::CabecalhoDeBlocosNaoReconhecido)?;
    let mut headers = None;
    for no in linhas {
        let linha = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBloco::CabecalhoDeBlocosNaoReconhecido)?;
        let th: Vec<_> = linha
            .children()
            .top()
            .iter()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
            .filter(|t| t.name() == "th")
            .collect();
        if !th.is_empty() {
            if headers.is_some() {
                return Err(ErroBloco::CabecalhoDeBlocosNaoReconhecido);
            }
            headers = Some(th.iter().map(|t| texto_tag(t, parser)).collect::<Vec<_>>());
        }
    }
    let h = headers.ok_or(ErroBloco::CabecalhoDeBlocosNaoReconhecido)?;
    for nome in ["Número", "Estado", "Descrição"] {
        if h.iter().filter(|v| *v == nome).count() != 1 {
            return Err(ErroBloco::CabecalhoDeBlocosNaoReconhecido);
        }
    }
    Ok(h)
}

fn declaracao(legenda: &str, observados: usize) -> Result<(usize, Option<Faixa>), ErroBloco> {
    let (inicio, resto) = legenda
        .split_once(" registros")
        .ok_or(ErroBloco::LegendaDeBlocosInvalida)?;
    let total = inicio
        .split_whitespace()
        .last()
        .ok_or(ErroBloco::LegendaDeBlocosInvalida)?
        .trim_start_matches('(')
        .replace('.', "")
        .parse::<usize>()
        .map_err(ErroBloco::TotalDaLegendaIlegivel)?;
    if let Some((_, intervalo)) = resto.split_once(" - ") {
        let (de, ate) = intervalo
            .split_once(" a ")
            .ok_or(ErroBloco::LegendaDeBlocosInvalida)?;
        let de = de
            .parse::<usize>()
            .map_err(ErroBloco::InicioDaFaixaIlegivel)?;
        let ate = ate
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<usize>()
            .map_err(ErroBloco::FimDaFaixaIlegivel)?;
        if de == 0 || ate < de || ate > total || ate - de + 1 != observados {
            return Err(ErroBloco::FaixaDeBlocosIncoerente);
        }
        return Ok((
            total,
            Some(Faixa {
                inicio: de,
                fim: ate,
            }),
        ));
    }
    if total != observados {
        return Err(ErroBloco::FaixaDeBlocosIncoerente);
    }
    let faixa = (total > 0).then_some(Faixa {
        inicio: 1,
        fim: total,
    });
    Ok((total, faixa))
}

/// Leitura de uma única página, com nomes de colunas, sem assumir ordem fixa.
/// O HTML da `UnB` (Todos `tests/paginas_reais/menu_blocos.html` @323eb01)
/// publica tblBlocos, caption, th Número/Estado/Descrição e href por número.
fn ler_linhas(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    nomes: &[String],
    tipo: TipoBloco,
) -> Result<(Vec<BlocoLido>, Vec<String>), ErroBloco> {
    let posicao = |s: &str| {
        nomes
            .iter()
            .position(|n| n == s)
            .ok_or(ErroBloco::CabecalhoDeBlocosNaoReconhecido)
    };
    let (indice_numero, indice_estado, indice_descricao) = (
        posicao("Número")?,
        posicao("Estado")?,
        posicao("Descrição")?,
    );
    let mut itens = Vec::new();
    let mut links = Vec::new();
    let mut vistos = HashSet::new();
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let linha = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBloco::LinhaDeBlocoForaDaForma)?;
        let td: Vec<_> = linha
            .children()
            .top()
            .iter()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
            .filter(|t| t.name() == "td")
            .collect();
        if td.is_empty() {
            continue;
        }
        if td.len() != nomes.len() {
            return Err(ErroBloco::LinhaDeBlocoForaDaForma);
        }
        let celula = |i: usize| td.get(i).copied().ok_or(ErroBloco::LinhaDeBlocoForaDaForma);
        let coluna_numero = celula(indice_numero)?;
        let numero = coluna_numero.inner_text(parser).into_owned();
        if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroBloco::NumeroDoBlocoInvalido);
        }
        if !vistos.insert(numero.clone()) {
            return Err(ErroBloco::NumeroDoBlocoRepetido);
        }
        let mut hrefs = coluna_numero
            .query_selector(parser, "a")
            .ok_or(ErroBloco::LinkDeBlocosNaoReconhecido)?
            .filter_map(|n| n.get(parser))
            .filter_map(tl::Node::as_tag)
            .map(|a| {
                valor_do_atributo(a, "href")
                    .map_err(ErroBloco::Atributo)
                    .and_then(|x| {
                        x.and_then(|s| decodificar_atributo(&s))
                            .ok_or(ErroBloco::LinkDeBlocosNaoReconhecido)
                    })
            });
        let href = hrefs
            .next()
            .ok_or(ErroBloco::LinkDeBlocosNaoReconhecido)??;
        if hrefs.next().is_some() {
            return Err(ErroBloco::LinkDeBlocosNaoReconhecido);
        }
        if !href.contains("acao=rel_bloco_protocolo_listar") {
            return Err(ErroBloco::LinkDeBlocosNaoReconhecido);
        }
        links.push(href);
        itens.push(BlocoLido {
            numero,
            tipo,
            estado: texto_tag(celula(indice_estado)?, parser),
            descricao: texto_tag(celula(indice_descricao)?, parser),
        });
    }
    Ok((itens, links))
}

pub(crate) fn ler_lista(
    html: &crate::tela_html::HtmlServido,
    tipo: TipoBloco,
) -> Result<ListaComLinks, ErroBloco> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblBlocos").map_err(ErroBloco::Atributo)?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroBloco::TabelaDeBlocosAusente),
        [no] => no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBloco::TabelaDeBlocosAusente)?,
        _ => return Err(ErroBloco::TabelasDeBlocosRepetidas),
    };
    if tabela.name() != "table" {
        return Err(ErroBloco::TabelaDeBlocosAusente);
    }
    let mut captions = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroBloco::LegendaDeBlocosInvalida)?;
    let legenda = captions
        .next()
        .and_then(|no| no.get(parser))
        .ok_or(ErroBloco::LegendaDeBlocosInvalida)?;
    if captions.next().is_some() {
        return Err(ErroBloco::LegendaDeBlocosInvalida);
    }
    let nomes = cabecalhos(tabela, parser)?;
    let (itens, links) = ler_linhas(tabela, parser, &nomes, tipo)?;
    let (total_declarado, faixa_observada) = declaracao(&texto_no(legenda, parser), itens.len())?;
    Ok(ListaComLinks {
        lista: ListaDeBlocos {
            tipo,
            total_declarado,
            faixa_observada,
            itens,
        },
        links,
    })
}

/// Valida o href de uma ação publicada antes de qualquer GET, sem expor URL
/// assinada. Proíbe `acao/infra_hash` duplicados, nomes não canônicos, caminhos
/// absolutos, URLs fora da instalação e qualquer fragmento.
pub(crate) fn resolver_link(
    instalacao: &Instalacao,
    url_da_tela: &str,
    href: &str,
    acao: AcaoDoSei,
    numero: Option<&str>,
) -> Result<String, ErroBloco> {
    let nao = || ErroBloco::LinkDeBlocosNaoReconhecido;
    let consulta = href.strip_prefix("controlador.php?").ok_or_else(nao)?;
    if !href.bytes().all(|b| (33..=126).contains(&b)) || href.contains(['#', '\\', '\'']) {
        return Err(nao());
    }
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroBloco::ConsultaDoLinkDeBloco)?;
    let acao_lida = consulta
        .unico("acao")
        .map_err(ErroBloco::ChaveDoLinkDeBloco)?;
    let hash = consulta
        .unico("infra_hash")
        .map_err(ErroBloco::ChaveDoLinkDeBloco)?;
    let ids: Vec<&str> = consulta.valores("id_bloco").collect();
    let id_conferido = match numero {
        Some(n) => ids == [n],
        None => ids.is_empty(),
    };
    if acao_lida != acao.como_na_url()
        || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
        || consulta.valores("acao_ajax").next().is_some()
        || !id_conferido
    {
        return Err(nao());
    }
    let base = url_da_tela.split('?').next().ok_or_else(nao)?;
    let (pasta, _) = base.rsplit_once('/').ok_or_else(nao)?;
    let url = format!("{pasta}/{href}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or_else(nao)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(nao());
    }
    Ok(url)
}

/// Seleciona o menu pelo valor exato de acao; nenhuma adivinhação de rota.
pub(crate) fn link_do_menu(
    instalacao: &Instalacao,
    url: &str,
    html: &crate::tela_html::HtmlServido,
    tipo: TipoBloco,
) -> Result<String, ErroBloco> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let mut candidatos = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(bruto) = valor_do_atributo(tag, "href").map_err(ErroBloco::Atributo)? else {
            continue;
        };
        let Some(href) = decodificar_atributo(&bruto) else {
            continue;
        };
        if menciona_acao(&href, tipo.acao()) {
            candidatos.push(href);
        }
    }
    match candidatos.as_slice() {
        [] => Err(ErroBloco::LinkDeBlocosAusente),
        [href] => resolver_link(instalacao, url, href, tipo.acao(), None),
        _ => Err(ErroBloco::LinksDeBlocosRepetidos),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fn linha(numero: &str, estado: &str, descricao: &str) -> String {
        format!(
            "<tr><td><input type='checkbox'/></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco={numero}&amp;infra_hash={HASH}'>{numero}</a></td><td>{descricao}</td><td>{estado}</td><td></td></tr>"
        )
    }
    fn pagina(linhas: &str, caption: &str) -> String {
        format!(
            "<table id='tblBlocos'><caption>{caption}</caption><tr><th></th><th>Número</th><th>Descrição</th><th>Estado</th><th>Ações</th></tr>{linhas}</table>"
        )
    }
    #[test]
    fn bloco_com_colunas_fora_de_ordem_e_numeros_opacos() {
        // Caso de Todos tests/paginas_reais/menu_blocos.html @323eb01
        let l = ler_lista(
            &crate::tela_html::HtmlServido::de_teste(&pagina(
                &linha("2195", "Gerado", "Teste de Bloco"),
                "Lista de Blocos (1 registros):",
            )),
            TipoBloco::Assinatura,
        )
        .unwrap();
        assert_eq!(l.lista.total_declarado, 1);
        assert_eq!(l.lista.itens[0].numero, "2195");
        assert_eq!(l.lista.itens[0].descricao, "Teste de Bloco");
        // A tela de listagem não mostra quantidade de itens: o campo não existe.
        assert!(
            !serde_json::to_string(&l.lista)
                .unwrap()
                .contains("quantidade_itens")
        );
        assert!(!serde_json::to_string(&l.lista).unwrap().contains(HASH));
    }
    #[test]
    fn duplicidade_ou_truncamento_nao_viram_lista_completa() {
        let um = linha("2195", "Gerado", "Exemplo");
        assert_eq!(
            ler_lista(
                &crate::tela_html::HtmlServido::de_teste(&pagina(
                    &(um.clone() + &um),
                    "Lista (2 registros):"
                )),
                TipoBloco::Interno
            )
            .err()
            .unwrap(),
            ErroBloco::NumeroDoBlocoRepetido
        );
        assert_eq!(
            ler_lista(
                &crate::tela_html::HtmlServido::de_teste(&pagina(&um, "Lista (3 registros):")),
                TipoBloco::Interno
            )
            .err()
            .unwrap(),
            ErroBloco::FaixaDeBlocosIncoerente
        );
    }
    #[test]
    fn faixa_de_pagina_e_observada_nao_inferida() {
        let l = ler_lista(
            &crate::tela_html::HtmlServido::de_teste(&pagina(
                &linha("2195", "Gerado", "X"),
                "Lista (100 registros - 51 a 51):",
            )),
            TipoBloco::Reuniao,
        )
        .unwrap();
        assert_eq!(
            l.lista.faixa_observada,
            Some(Faixa {
                inicio: 51,
                fim: 51
            })
        );
    }
    #[test]
    fn tipo_certo_e_hash_unico_sao_obrigatorios() {
        let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
        let base = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar";
        let href = format!("controlador.php?acao=bloco_interno_listar&infra_hash={HASH}");
        assert!(
            resolver_link(
                &instalacao,
                base,
                &href,
                AcaoDoSei::BlocoInternoListar,
                None
            )
            .is_ok()
        );
        assert_eq!(
            resolver_link(
                &instalacao,
                base,
                &format!("{href}&infra_hash={HASH}"),
                AcaoDoSei::BlocoInternoListar,
                None
            ),
            Err(ErroBloco::ChaveDoLinkDeBloco(
                crate::consulta_do_controlador::ErroChaveDaConsulta::Repetida {
                    chave: "infra_hash"
                }
            ))
        );
        assert_eq!(
            resolver_link(
                &instalacao,
                base,
                "https://outra.test/controlador.php?acao=bloco_interno_listar",
                AcaoDoSei::BlocoInternoListar,
                None
            ),
            Err(ErroBloco::LinkDeBlocosNaoReconhecido)
        );
    }

    #[test]
    fn link_de_bloco_com_chave_repetida_ou_par_sem_igual_tem_a_causa() {
        let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
        let base = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar";
        let link = |consulta: String| {
            let href = format!("controlador.php?{consulta}");
            resolver_link(
                &instalacao,
                base,
                &href,
                AcaoDoSei::BlocoInternoListar,
                None,
            )
        };
        assert_eq!(
            link(format!(
                "acao=bloco_interno_listar&acao=bloco_interno_listar&infra_hash={HASH}"
            )),
            Err(ErroBloco::ChaveDoLinkDeBloco(
                ErroChaveDaConsulta::Repetida { chave: "acao" }
            ))
        );
        assert_eq!(
            link(format!("acao=bloco_interno_listar&lixo&infra_hash={HASH}")),
            Err(ErroBloco::ConsultaDoLinkDeBloco(
                ErroConsultaDoControlador::ParSemIgual
            ))
        );
        assert_eq!(
            link(format!(
                "acao=bloco_interno_listar&infra_hash={HASH}&id_bloco=5"
            )),
            Err(ErroBloco::LinkDeBlocosNaoReconhecido)
        );
    }
}
