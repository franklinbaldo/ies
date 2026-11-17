//! Leitores puros de blocos do SEI: tabela por cabeçalho, sem JS nem escrita.
//! Oráculo de cenários: Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco_estado::EstadoDoBloco;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroAcaoDaUrl, ErroConsultaDoControlador, ErroParametroDaUrl,
    menciona_acao,
};
use crate::instalacao::Instalacao;
use crate::legenda::{ErroLegenda, Faixa, Legenda, ler_legenda};
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
    FaixaDeBlocosIncoerente,
    PaginaDeBlocosNaoImplementada,
    BlocoNaoEncontradoNaPagina,
    CabecalhoDeBlocosNaoReconhecido,
    Legenda(#[from] ErroLegenda),
    /// A consulta do link de bloco não se deixa ler em pares.
    ConsultaDoLinkDeBloco(#[source] ErroConsultaDoControlador),
    /// `infra_hash` do link de bloco ausente ou repetido.
    ChaveDoLinkDeBloco(#[source] ErroParametroDaUrl),
    AcaoDoLinkDeBloco(#[source] ErroAcaoDaUrl),
    /// A coluna Estado traz um texto fora do conjunto que o SEI publica.
    EstadoDoBlocoDesconhecido(String),
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
    pub(crate) estado: EstadoDoBloco,
    pub(crate) descricao: String,
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

fn texto_do_elemento(elemento: crate::tela_html::Elemento<'_>) -> String {
    elemento
        .texto()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Texto fora do conjunto de estados não passa: vira erro com o texto observado.
fn estado_da_celula(celula: crate::tela_html::Elemento<'_>) -> Result<EstadoDoBloco, ErroBloco> {
    let texto = texto_do_elemento(celula);
    EstadoDoBloco::ler(&texto).ok_or(ErroBloco::EstadoDoBlocoDesconhecido(texto))
}

fn cabecalhos(tabela: crate::tela_html::Elemento<'_>) -> Result<Vec<String>, ErroBloco> {
    let linhas = tabela.elementos("tr");
    let mut headers = None;
    for no in linhas {
        let th: Vec<_> = no.filhos().filter(|t| t.nome() == "th").collect();
        if !th.is_empty() {
            if headers.is_some() {
                return Err(ErroBloco::CabecalhoDeBlocosNaoReconhecido);
            }
            headers = Some(
                th.iter()
                    .copied()
                    .map(texto_do_elemento)
                    .collect::<Vec<_>>(),
            );
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
    let Legenda { total, faixa } = ler_legenda(legenda)?;
    let Some(Faixa { inicio, fim }) = faixa else {
        if total != observados {
            return Err(ErroBloco::FaixaDeBlocosIncoerente);
        }
        return Ok((
            total,
            (total > 0).then_some(Faixa {
                inicio: 1,
                fim: total,
            }),
        ));
    };
    if inicio == 0 || fim < inicio || fim > total || fim - inicio + 1 != observados {
        return Err(ErroBloco::FaixaDeBlocosIncoerente);
    }
    Ok((total, Some(Faixa { inicio, fim })))
}

/// Leitura de uma única página, com nomes de colunas, sem assumir ordem fixa.
/// O HTML da `UnB` (Todos `tests/paginas_reais/menu_blocos.html` @323eb01)
/// publica tblBlocos, caption, th Número/Estado/Descrição e href por número.
fn ler_linhas(
    tabela: crate::tela_html::Elemento<'_>,
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
    for no in tabela.elementos("tr") {
        let td: Vec<_> = no.filhos().filter(|t| t.nome() == "td").collect();
        if td.is_empty() {
            continue;
        }
        if td.len() != nomes.len() {
            return Err(ErroBloco::LinhaDeBlocoForaDaForma);
        }
        let celula = |i: usize| td.get(i).copied().ok_or(ErroBloco::LinhaDeBlocoForaDaForma);
        let coluna_numero = celula(indice_numero)?;
        let numero = coluna_numero.texto();
        if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroBloco::NumeroDoBlocoInvalido);
        }
        if !vistos.insert(numero.clone()) {
            return Err(ErroBloco::NumeroDoBlocoRepetido);
        }
        let mut hrefs = coluna_numero.elementos("a").map(|a| {
            a.atributo("href")
                .map(str::to_owned)
                .ok_or(ErroBloco::LinkDeBlocosNaoReconhecido)
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
            estado: estado_da_celula(celula(indice_estado)?)?,
            descricao: texto_do_elemento(celula(indice_descricao)?),
        });
    }
    Ok((itens, links))
}

const LISTA_VAZIA: &str = "Nenhum registro encontrado.";

/// Verdadeiro só para a forma observada no acervo: a área da tabela tem como
/// único conteúdo um rótulo com a marca de vazio, e `tblBlocos` não existe.
fn lista_vazia(dom: &crate::tela_html::Documento) -> bool {
    let areas = dom.com_id("divInfraAreaTabela").collect::<Vec<_>>();
    let [area] = areas.as_slice() else {
        return false;
    };
    let area = *area;
    let mut rotulos = area.elementos("label");
    let rotulo = rotulos.next();
    rotulos.next().is_none()
        && rotulo.is_some_and(|r| texto_do_elemento(r) == LISTA_VAZIA)
        && texto_do_elemento(area) == LISTA_VAZIA
}

pub(crate) fn ler_lista(
    html: &crate::tela_html::HtmlServido,
    tipo: TipoBloco,
) -> Result<ListaComLinks, ErroBloco> {
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblBlocos").collect::<Vec<_>>();
    if tabelas.is_empty() && lista_vazia(&dom) {
        return Ok(ListaComLinks {
            lista: ListaDeBlocos {
                tipo,
                total_declarado: 0,
                faixa_observada: None,
                itens: Vec::new(),
            },
            links: Vec::new(),
        });
    }
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroBloco::TabelaDeBlocosAusente),
        [no] => *no,
        _ => return Err(ErroBloco::TabelasDeBlocosRepetidas),
    };
    if tabela.nome() != "table" {
        return Err(ErroBloco::TabelaDeBlocosAusente);
    }
    let mut captions = tabela.elementos("caption");
    let legenda = captions.next().ok_or(ErroBloco::LegendaDeBlocosInvalida)?;
    if captions.next().is_some() {
        return Err(ErroBloco::LegendaDeBlocosInvalida);
    }
    let nomes = cabecalhos(tabela)?;
    let (itens, links) = ler_linhas(tabela, &nomes, tipo)?;
    let (total_declarado, faixa_observada) = declaracao(&texto_do_elemento(legenda), itens.len())?;
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
    let acao_lida = consulta.acao().map_err(ErroBloco::AcaoDoLinkDeBloco)?;
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
    let base = url_da_tela
        .split_once('?')
        .map_or(url_da_tela, |(antes, _)| antes);
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
    let dom = crate::tela_html::arvore(html);
    let mut candidatos = Vec::new();
    for no in dom.elementos("a") {
        let Some(bruto) = no.atributo("href") else {
            continue;
        };
        if menciona_acao(bruto, tipo.acao()) {
            candidatos.push(bruto);
        }
    }
    match candidatos.as_slice() {
        [] => Err(ErroBloco::LinkDeBlocosAusente),
        [href] => resolver_link(instalacao, url, href, tipo.acao(), None),
        _ => Err(ErroBloco::LinksDeBlocosRepetidos),
    }
}

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests;
