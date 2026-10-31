//! Leitura estritamente offline do histórico Web de um processo.
//! Não confundir histórico parcial com a representação integral do processo.
use crate::processo::{Andamento, DetalheProcesso, NumeroProcesso};

/// Condições específicas do parser, independentes de mensagens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroHistorico {
    HtmlInvalido,
    TituloAusente,
    TituloInvalido,
    NumeroInvalido,
    NumeroDoHistoricoDiferenteDoSolicitado,
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
            Self::NumeroInvalido => "numero_invalido",
            Self::NumeroDoHistoricoDiferenteDoSolicitado => {
                "numero_do_historico_diferente_do_solicitado"
            }
            Self::HistoricoAusente => "historico_ausente",
            Self::HistoricoInvalido => "historico_invalido",
            Self::LinhaInvalida => "linha_invalida",
            Self::HistoricoVazio => "historico_vazio",
        }
    }

    pub fn mensagem(self) -> &'static str {
        match self {
            Self::HtmlInvalido => "HTML do histórico não pôde ser interpretado",
            Self::TituloAusente => "título do histórico ausente",
            Self::TituloInvalido => "o título não informa que esta é a página de histórico",
            Self::NumeroInvalido => "número de processo informado não é aceito",
            Self::NumeroDoHistoricoDiferenteDoSolicitado => {
                "número mostrado no título do histórico é diferente do solicitado"
            }
            Self::HistoricoAusente => "tabela do histórico não encontrada",
            Self::HistoricoInvalido => "tabela do histórico não possui a estrutura esperada",
            Self::LinhaInvalida => "linha do histórico sem campos obrigatórios",
            Self::HistoricoVazio => "nenhum andamento foi encontrado no histórico",
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
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroHistorico::HtmlInvalido)?;
    let parser = dom.parser();
    let titulo = dom
        .query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or(ErroHistorico::TituloAusente)?;
    let (_, protocolo) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHistorico::TituloInvalido)?;
    let numero = NumeroProcesso::novo(protocolo).map_err(|_| ErroHistorico::NumeroInvalido)?;
    Ok(numero.as_str().to_owned())
}

pub fn ler_historico(html: &str, numero: &str) -> Result<DetalheProcesso, ErroHistorico> {
    let numero = NumeroProcesso::novo(numero).map_err(|_| ErroHistorico::NumeroInvalido)?;
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroHistorico::HtmlInvalido)?;
    let parser = dom.parser();
    let titulo = dom
        .query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or(ErroHistorico::TituloAusente)?;
    // O protocolo é opaco. Não fragmentar por pontuação nem remover zeros.
    let (_, mostrado) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHistorico::TituloInvalido)?;
    if mostrado != numero.as_str() {
        return Err(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado);
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
        assert_eq!(resultado.atuacoes, []);
        assert!(resultado.id_interno.is_none());
    }

    #[test]
    fn historico_aceita_protocolo_opaco_sem_mascara() {
        for protocolo in [
            "23106.000001/2026-01",
            "550e8400-e29b-41d4-a716-446655440000",
            "000123",
            "AB-C",
        ] {
            let html = fixture(protocolo);
            assert_eq!(numero_do_historico(&html).unwrap(), protocolo);
            assert_eq!(
                ler_historico(&html, protocolo).unwrap().numero.as_str(),
                protocolo
            );
        }
    }

    #[test]
    fn historico_nao_confunde_prefixos_de_protocolo() {
        let html = fixture("AB-C");
        assert_eq!(
            ler_historico(&html, "ABC").unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
        assert_eq!(
            ler_historico(&fixture("000123"), "123").unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
    }

    #[test]
    fn recusa_outro_processo_e_tabela_incompleta() {
        assert_eq!(
            ler_historico(&fixture("202600005002487"), "202600005002488").unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
        let invalido = "<title>Histórico do Processo 202600005002487</title>        <table id='tblHistorico'><tr><td>12/07/2026</td></tr></table>";
        assert_eq!(
            ler_historico(invalido, "202600005002487").unwrap_err(),
            ErroHistorico::LinhaInvalida
        );
    }
}
