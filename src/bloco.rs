//! Leitores puros de blocos do SEI: tabela por cabeçalho, sem JS nem escrita.
//! Oráculo de cenários: Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
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
    pub(crate) const fn acao(self) -> &'static str {
        match self {
            Self::Assinatura => "bloco_assinatura_listar",
            Self::Interno => "bloco_interno_listar",
            Self::Reuniao => "bloco_reuniao_listar",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBloco {
    FaixaDeBlocosIncoerente,
    PaginaDeBlocosNaoImplementada,
    Html(#[from] ErroHtml),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BlocoLido {
    pub(crate) numero: String,
    pub(crate) tipo: TipoBloco,
    pub(crate) estado: String,
    pub(crate) descricao: String,
    /// A listagem não informa o número de documentos/processos do bloco.
    /// None significa não observado, não zero.
    pub(crate) quantidade_itens: Option<usize>,
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
            _ => Err(ErroHtml::ausente("bloco_nao_encontrado_na_pagina")
                .com_mensagem("bloco não observado nesta página da unidade")
                .into()),
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
        .ok_or(ErroHtml::valor("cabecalho_de_blocos_nao_reconhecido"))?;
    let mut headers = None;
    for no in linhas {
        let linha = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("cabecalho_de_blocos_nao_reconhecido"))?;
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
                return Err(ErroHtml::valor("cabecalho_de_blocos_nao_reconhecido").into());
            }
            headers = Some(th.iter().map(|t| texto_tag(t, parser)).collect::<Vec<_>>());
        }
    }
    let h = headers.ok_or(ErroHtml::valor("cabecalho_de_blocos_nao_reconhecido"))?;
    for nome in ["Número", "Estado", "Descrição"] {
        if h.iter().filter(|v| *v == nome).count() != 1 {
            return Err(ErroHtml::valor("cabecalho_de_blocos_nao_reconhecido").into());
        }
    }
    Ok(h)
}

fn declaracao(legenda: &str, observados: usize) -> Result<(usize, Option<Faixa>), ErroBloco> {
    let (inicio, resto) = legenda
        .split_once(" registros")
        .ok_or(ErroHtml::valor("legenda_de_blocos_invalida"))?;
    let total = inicio
        .split_whitespace()
        .last()
        .ok_or(ErroHtml::valor("legenda_de_blocos_invalida"))?
        .trim_start_matches('(')
        .replace('.', "")
        .parse::<usize>()
        .map_err(|_| ErroHtml::valor("legenda_de_blocos_invalida"))?;
    if let Some((_, intervalo)) = resto.split_once(" - ") {
        let (de, ate) = intervalo
            .split_once(" a ")
            .ok_or(ErroHtml::valor("legenda_de_blocos_invalida"))?;
        let de = de
            .parse::<usize>()
            .map_err(|_| ErroHtml::valor("legenda_de_blocos_invalida"))?;
        let ate = ate
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| ErroHtml::valor("legenda_de_blocos_invalida"))?;
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
            .expect("cabeçalho conferido")
    };
    let (indice_numero, indice_estado, indice_descricao) =
        (posicao("Número"), posicao("Estado"), posicao("Descrição"));
    let mut itens = Vec::new();
    let mut links = Vec::new();
    let mut vistos = HashSet::new();
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let linha = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("linha_de_bloco_fora_da_forma"))?;
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
            return Err(ErroHtml::valor("linha_de_bloco_fora_da_forma").into());
        }
        let numero = td[indice_numero].inner_text(parser).into_owned();
        if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroHtml::valor("numero_do_bloco_invalido").into());
        }
        if !vistos.insert(numero.clone()) {
            return Err(ErroHtml::repetido("numero_do_bloco_repetido").into());
        }
        let mut hrefs = td[indice_numero]
            .query_selector(parser, "a")
            .ok_or(
                ErroHtml::valor("link_de_blocos_nao_reconhecido")
                    .com_mensagem("link de blocos ausente ou ambíguo na tela"),
            )?
            .filter_map(|n| n.get(parser))
            .filter_map(tl::Node::as_tag)
            .map(|a| {
                valor_do_atributo(a, "href")
                    .map_err(|e| {
                        ErroBloco::Html(
                            ErroHtml::atributo("link_de_blocos_nao_reconhecido", e)
                                .com_mensagem("link de blocos ausente ou ambíguo na tela"),
                        )
                    })
                    .and_then(|x| {
                        x.and_then(|s| decodificar_atributo(&s))
                            .ok_or(ErroBloco::Html(
                                ErroHtml::valor("link_de_blocos_nao_reconhecido")
                                    .com_mensagem("link de blocos ausente ou ambíguo na tela"),
                            ))
                    })
            });
        let href = hrefs.next().ok_or(
            ErroHtml::valor("link_de_blocos_nao_reconhecido")
                .com_mensagem("link de blocos ausente ou ambíguo na tela"),
        )??;
        if hrefs.next().is_some() {
            return Err(ErroHtml::valor("link_de_blocos_nao_reconhecido")
                .com_mensagem("link de blocos ausente ou ambíguo na tela")
                .into());
        }
        if !href.contains("acao=rel_bloco_protocolo_listar") {
            return Err(ErroHtml::valor("link_de_blocos_nao_reconhecido")
                .com_mensagem("link de blocos ausente ou ambíguo na tela")
                .into());
        }
        links.push(href);
        itens.push(BlocoLido {
            numero,
            tipo,
            estado: texto_tag(td[indice_estado], parser),
            descricao: texto_tag(td[indice_descricao], parser),
            quantidade_itens: None,
        });
    }
    Ok((itens, links))
}

pub(crate) fn ler_lista(html: &str, tipo: TipoBloco) -> Result<ListaComLinks, ErroBloco> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_de_blocos_invalido"))?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblBlocos")
        .map_err(|e| ErroHtml::atributo("html_de_blocos_invalido", e))?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroHtml::ausente("tabela_de_blocos_ausente").into()),
        [no] => no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::ausente("tabela_de_blocos_ausente"))?,
        _ => return Err(ErroHtml::repetido("tabelas_de_blocos_repetidas").into()),
    };
    if tabela.name() != "table" {
        return Err(ErroHtml::ausente("tabela_de_blocos_ausente").into());
    }
    let mut captions = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroHtml::valor("legenda_de_blocos_invalida"))?;
    let legenda = captions
        .next()
        .and_then(|no| no.get(parser))
        .ok_or(ErroHtml::valor("legenda_de_blocos_invalida"))?;
    if captions.next().is_some() {
        return Err(ErroHtml::valor("legenda_de_blocos_invalida").into());
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
    acao: &str,
    numero: Option<&str>,
) -> Result<String, ErroBloco> {
    let nao = ErroBloco::Html(
        ErroHtml::valor("link_de_blocos_nao_reconhecido")
            .com_mensagem("link de blocos ausente ou ambíguo na tela"),
    );
    let consulta = href.strip_prefix("controlador.php?").ok_or(nao)?;
    if !href.bytes().all(|b| (33..=126).contains(&b)) || href.contains(['#', '\\', '\'']) {
        return Err(nao);
    }
    let (mut acoes, mut hashes, mut ids) = (0, 0, 0);
    for par in consulta.split('&') {
        let (chave, valor) = par.split_once('=').ok_or(nao)?;
        conferir_chave_de_consulta(chave).map_err(|_| nao)?;
        conferir_valor_de_consulta(valor).map_err(|_| nao)?;
        match chave {
            "acao" if valor == acao => acoes += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "id_bloco" if numero == Some(valor) => ids += 1,
            "acao" | "infra_hash" | "id_bloco" | "acao_ajax" => return Err(nao),
            _ => {}
        }
    }
    if acoes != 1 || hashes != 1 || (numero.is_some() && ids != 1) || (numero.is_none() && ids != 0)
    {
        return Err(nao);
    }
    let base = url_da_tela.split('?').next().ok_or(nao)?;
    let (pasta, _) = base.rsplit_once('/').ok_or(nao)?;
    let url = format!("{pasta}/{href}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or(nao)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(nao);
    }
    Ok(url)
}

/// Seleciona o menu pelo valor exato de acao; nenhuma adivinhação de rota.
pub(crate) fn link_do_menu(
    instalacao: &Instalacao,
    url: &str,
    html: &str,
    tipo: TipoBloco,
) -> Result<String, ErroBloco> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_de_blocos_invalido"))?;
    let parser = dom.parser();
    let mut candidatos = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(bruto) = valor_do_atributo(tag, "href")
            .map_err(|e| ErroHtml::atributo("html_de_blocos_invalido", e))?
        else {
            continue;
        };
        let Some(href) = decodificar_atributo(&bruto) else {
            continue;
        };
        if href
            .split_once('?')
            .is_some_and(|(_, q)| q.split('&').any(|p| p == format!("acao={}", tipo.acao())))
        {
            candidatos.push(href);
        }
    }
    match candidatos.as_slice() {
        [] => Err(ErroHtml::ausente("link_de_blocos_ausente")
            .com_mensagem("link de blocos ausente ou ambíguo na tela")
            .into()),
        [href] => resolver_link(instalacao, url, href, tipo.acao(), None),
        _ => Err(ErroHtml::repetido("links_de_blocos_repetidos")
            .com_mensagem("link de blocos ausente ou ambíguo na tela")
            .into()),
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
            &pagina(
                &linha("2195", "Gerado", "Teste de Bloco"),
                "Lista de Blocos (1 registros):",
            ),
            TipoBloco::Assinatura,
        )
        .unwrap();
        assert_eq!(l.lista.total_declarado, 1);
        assert_eq!(l.lista.itens[0].numero, "2195");
        assert_eq!(l.lista.itens[0].descricao, "Teste de Bloco");
        assert_eq!(l.lista.itens[0].quantidade_itens, None);
        assert!(
            serde_json::to_string(&l.lista)
                .unwrap()
                .contains("\"quantidade_itens\":null")
        );
        assert!(!serde_json::to_string(&l.lista).unwrap().contains(HASH));
    }
    #[test]
    fn duplicidade_ou_truncamento_nao_viram_lista_completa() {
        let um = linha("2195", "Gerado", "Exemplo");
        assert_eq!(
            ler_lista(
                &pagina(&(um.clone() + &um), "Lista (2 registros):"),
                TipoBloco::Interno
            )
            .err()
            .unwrap(),
            ErroBloco::Html(ErroHtml::repetido("numero_do_bloco_repetido"))
        );
        assert_eq!(
            ler_lista(&pagina(&um, "Lista (3 registros):"), TipoBloco::Interno)
                .err()
                .unwrap(),
            ErroBloco::FaixaDeBlocosIncoerente
        );
    }
    #[test]
    fn faixa_de_pagina_e_observada_nao_inferida() {
        let l = ler_lista(
            &pagina(
                &linha("2195", "Gerado", "X"),
                "Lista (100 registros - 51 a 51):",
            ),
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
        assert!(resolver_link(&instalacao, base, &href, "bloco_interno_listar", None).is_ok());
        assert_eq!(
            resolver_link(
                &instalacao,
                base,
                &format!("{href}&infra_hash={HASH}"),
                "bloco_interno_listar",
                None
            ),
            Err(ErroBloco::Html(ErroHtml::valor(
                "link_de_blocos_nao_reconhecido"
            )))
        );
        assert_eq!(
            resolver_link(
                &instalacao,
                base,
                "https://outra.test/controlador.php?acao=bloco_interno_listar",
                "bloco_interno_listar",
                None
            ),
            Err(ErroBloco::Html(ErroHtml::valor(
                "link_de_blocos_nao_reconhecido"
            )))
        );
    }
}
