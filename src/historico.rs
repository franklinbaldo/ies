//! Leitura estritamente offline do histórico Web de um processo.
//! Não confundir histórico parcial com a representação integral do processo.
use crate::processo::{Andamento, DetalheProcesso, NumeroProcesso};

/// Condições específicas do parser, independentes de mensagens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroHistorico {
    HtmlInvalido,
    TituloAusente,
    TituloInvalido,
    NumeroAmbiguo,
    NumeroInvalido,
    ProcessoDivergente,
    HistoricoAusente,
    HistoricoInvalido,
    LinhaInvalida,
    HistoricoVazio,
}

impl ErroHistorico {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlInvalido => "html_invalido",
            Self::TituloAusente => "titulo_ausente",
            Self::TituloInvalido => "titulo_invalido",
            Self::NumeroAmbiguo => "numero_ambiguo",
            Self::NumeroInvalido => "numero_invalido",
            Self::ProcessoDivergente => "processo_divergente",
            Self::HistoricoAusente => "historico_ausente",
            Self::HistoricoInvalido => "historico_invalido",
            Self::LinhaInvalida => "linha_invalida",
            Self::HistoricoVazio => "historico_vazio",
        }
    }
}

fn texto(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    let valor = no.inner_text(parser);
    valor
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn numero_do_historico(html: &str) -> Result<String, ErroHistorico> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroHistorico::HtmlInvalido)?;
    let parser = dom.parser();
    let titulo = dom
        .query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or(ErroHistorico::TituloAusente)?;
    let (_, depois) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHistorico::TituloInvalido)?;
    let candidatos: Vec<_> = depois
        .split_whitespace()
        .filter(|v| {
            !v.is_empty()
                && v.bytes()
                    .all(|b| b.is_ascii_digit() || b == b'.' || b == b'/' || b == b'-')
        })
        .collect();
    if candidatos.len() != 1 {
        return Err(ErroHistorico::NumeroAmbiguo);
    }
    Ok(candidatos[0].to_owned())
}

pub fn ler_historico(html: &str, numero: &str) -> Result<DetalheProcesso, ErroHistorico> {
    let numero = NumeroProcesso::novo(numero).map_err(|_| ErroHistorico::NumeroInvalido)?;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroHistorico::HtmlInvalido)?;
    let parser = dom.parser();
    let titulo = dom
        .query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or(ErroHistorico::TituloAusente)?;
    // Não converter um histórico de outro processo numa resposta para o NUP pedido.
    if !titulo
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|item| item == numero.as_str())
    {
        return Err(ErroHistorico::ProcessoDivergente);
    }
    let tabela = dom
        .get_element_by_id("tblHistorico")
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHistorico::HistoricoAusente)?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut nodes| nodes.next().is_some())
    {
        return Err(ErroHistorico::HistoricoInvalido);
    }
    let mut detalhe = DetalheProcesso::novo(numero);
    let linhas = tabela
        .query_selector(parser, "tr")
        .ok_or(ErroHistorico::HistoricoInvalido)?;
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHistorico::HistoricoInvalido)?;
        let cells: Vec<String> = tag
            .query_selector(parser, "td")
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(parser))
            .map(|no| texto(no, parser))
            .collect();
        if cells.is_empty() {
            continue;
        }
        if cells.len() != 4 || cells[0].is_empty() || cells[3].is_empty() {
            return Err(ErroHistorico::LinhaInvalida);
        }
        detalhe.andamentos.push(Andamento {
            ocorrido_em: Some(cells[0].clone()),
            unidade: (!cells[1].is_empty()).then(|| cells[1].clone()),
            usuario: (!cells[2].is_empty()).then(|| cells[2].clone()),
            descricao: cells[3].clone(),
        });
    }
    if detalhe.andamentos.is_empty() {
        return Err(ErroHistorico::HistoricoVazio);
    }
    Ok(detalhe)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(numero: &str) -> String {
        format!(
            "<title>SEI - Histórico do Processo {numero}&nbsp;</title>             <table id='tblHistorico'><tr><th>Data/Hora</th></tr>             <tr class='andamentoAberto'><td>12/07/2026 17:49</td>             <td>UNIDADE A</td><td>usuario</td>             <td>Processo recebido na unidade</td></tr></table>"
        )
    }

    #[test]
    fn interpreta_historico_sem_inventar_identificadores() {
        let resultado = ler_historico(&fixture("202600005002487"), "202600005002487").unwrap();
        assert_eq!(resultado.andamentos.len(), 1);
        assert_eq!(resultado.andamentos[0].usuario.as_deref(), Some("usuario"));
        assert!(resultado.atuacoes.is_empty());
        assert!(resultado.id_interno.is_none());
    }

    #[test]
    fn recusa_outro_processo_e_tabela_incompleta() {
        assert_eq!(
            ler_historico(&fixture("202600005002487"), "202600005002488").unwrap_err(),
            ErroHistorico::ProcessoDivergente
        );
        let invalido = "<title>Histórico do Processo 202600005002487</title>        <table id='tblHistorico'><tr><td>12/07/2026</td></tr></table>";
        assert_eq!(
            ler_historico(invalido, "202600005002487").unwrap_err(),
            ErroHistorico::LinhaInvalida
        );
    }
}
