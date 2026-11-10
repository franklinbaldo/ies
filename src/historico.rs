//! Leitura estritamente offline do histórico Web de um processo.
//! Não confundir histórico parcial com a representação integral do processo.
use crate::erro_html::ErroHtml;
use crate::html_atributos::nos_com_id;
use crate::processo::{Andamento, DetalheProcesso, NumeroProcesso};

/// Condições específicas do parser, independentes de mensagens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum ErroHistorico {
    NumeroDoHistoricoDiferenteDoSolicitado,
    HistoricoVazio,
    #[causa(transparente)]
    Html(ErroHtml),
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

/// O título do SVG não pode assumir a identidade do processo.
fn titulo_do_historico(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
) -> Result<String, ErroHistorico> {
    let mut cabecalhos = dom.query_selector("head").into_iter().flatten();
    let titulos: Vec<_> = if let Some(cabecalho) = cabecalhos.next() {
        if cabecalhos.next().is_some() {
            return Err(ErroHtml::repetido("cabecalhos_html_do_historico_repetidos").into());
        }
        let tag = cabecalho
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::ausente("titulo_ausente"))?;
        tag.query_selector(parser, "title")
            .into_iter()
            .flatten()
            .collect()
    } else {
        dom.query_selector("title").into_iter().flatten().collect()
    };
    let mut titulos = titulos.into_iter();
    let primeiro = titulos.next().ok_or(ErroHtml::ausente("titulo_ausente"))?;
    if titulos.next().is_some() {
        return Err(ErroHtml::repetido("titulos_do_historico_repetidos").into());
    }
    primeiro
        .get(parser)
        .map(|no| texto(no, parser))
        .ok_or(ErroHistorico::Html(ErroHtml::ausente("titulo_ausente")))
}

#[cfg(test)]
pub fn numero_do_historico(html: &str) -> Result<String, ErroHistorico> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_do_historico_invalido"))?;
    let parser = dom.parser();
    let titulo = titulo_do_historico(&dom, parser)?;
    let (_, protocolo) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHtml::valor("titulo_invalido"))?;
    let numero = NumeroProcesso::novo(protocolo)
        .map_err(|_| ErroHtml::valor("numero_do_historico_invalido"))?;
    Ok(numero.as_str().to_owned())
}

pub fn ler_historico(html: &str, numero: &str) -> Result<DetalheProcesso, ErroHistorico> {
    let numero = NumeroProcesso::novo(numero)
        .map_err(|_| ErroHtml::valor("numero_do_historico_invalido"))?;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_do_historico_invalido"))?;
    let parser = dom.parser();
    let titulo = titulo_do_historico(&dom, parser)?;
    // O protocolo é opaco. Não fragmentar por pontuação nem remover zeros.
    let (_, mostrado) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHtml::valor("titulo_invalido"))?;
    if mostrado != numero.as_str() {
        return Err(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado);
    }
    // O primeiro ID não é suficiente: tabelas duplicadas podem ocultar andamentos.
    let candidatas = nos_com_id(&dom, "tblHistorico")
        .map_err(|e| ErroHtml::atributo("html_do_historico_invalido", e))?;
    let primeira = match candidatas.as_slice() {
        [] => return Err(ErroHtml::ausente("historico_ausente").into()),
        [primeira] => *primeira,
        _ => return Err(ErroHtml::repetido("tabelas_do_historico_repetidas").into()),
    };
    let tabela = primeira
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHtml::valor("historico_invalido"))?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut nodes| nodes.next().is_some())
    {
        return Err(ErroHtml::valor("historico_invalido").into());
    }
    let mut detalhe = DetalheProcesso::novo(numero);
    let linhas = tabela
        .query_selector(parser, "tr")
        .ok_or(ErroHtml::valor("historico_invalido"))?;
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("historico_invalido"))?;
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
            return Err(ErroHtml::valor("linha_invalida").into());
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
    fn booleano_antes_do_id_nao_esconde_a_tabela_do_historico() {
        let html =
            crate::html_atributos::com_booleano_antes_dos_atributos(&fixture("202600005002487"));
        let resultado = ler_historico(&html, "202600005002487").unwrap();
        assert_eq!(resultado.andamentos.len(), 1);
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
            ErroHistorico::Html(ErroHtml::valor("linha_invalida"))
        );
    }

    #[test]
    fn duas_tabelas_de_historico_falham_sem_omitir_andamentos() {
        let numero = "000123";
        let html = format!(
            "{}<table id='tblHistorico'><tr><td>13/07/2026</td><td>U</td><td>X</td><td>Outro andamento</td></tr></table>",
            fixture(numero)
        );
        let erro = ler_historico(&html, numero).unwrap_err();
        assert_eq!(
            erro,
            ErroHistorico::Html(ErroHtml::repetido("tabelas_do_historico_repetidas"))
        );
    }

    #[test]
    fn tabela_com_id_de_historico_em_div_e_table_e_ambigua() {
        let numero = "000123";
        let html = format!("<div id='tblHistorico'>navegação</div>{}", fixture(numero));
        assert_eq!(
            ler_historico(&html, numero).unwrap_err(),
            ErroHistorico::Html(ErroHtml::repetido("tabelas_do_historico_repetidas"))
        );
    }

    #[test]
    fn svg_no_body_nao_substitui_identidade_do_processo() {
        let html = "<html><head><title>SEI - Histórico do Processo 000123</title></head>
            <body><svg><title>Histórico do Processo INTRUSO</title></svg>
            <table id='tblHistorico'>
              <tr><td>12/07/2026</td><td>U</td><td>X</td><td>Movimento</td></tr>
            </table></body></html>";
        assert_eq!(numero_do_historico(html).unwrap(), "000123");
        assert_eq!(ler_historico(html, "000123").unwrap().andamentos.len(), 1);
    }

    #[test]
    fn svg_nao_corrige_head_sem_titulo() {
        let html = "<html><head></head><body><svg>
            <title>SEI - Histórico do Processo 000123</title>
            </svg></body></html>";
        assert_eq!(
            numero_do_historico(html),
            Err(ErroHistorico::Html(ErroHtml::ausente("titulo_ausente")))
        );
        assert_eq!(
            ler_historico(html, "000123"),
            Err(ErroHistorico::Html(ErroHtml::ausente("titulo_ausente")))
        );
    }

    #[test]
    fn recusa_titulos_ou_cabecalhos_html_repetidos() {
        let titulos = "<html><head>
            <title>SEI - Histórico do Processo 000123</title>
            <title>SEI - Histórico do Processo 000124</title></head></html>";
        assert_eq!(
            numero_do_historico(titulos),
            Err(ErroHistorico::Html(ErroHtml::repetido(
                "titulos_do_historico_repetidos"
            )))
        );
        assert_eq!(
            ler_historico(titulos, "000123"),
            Err(ErroHistorico::Html(ErroHtml::repetido(
                "titulos_do_historico_repetidos"
            )))
        );
        let cabecalhos = "<html>
            <head><title>SEI - Histórico do Processo 000123</title></head>
            <head><title>SEI - Histórico do Processo 000124</title></head></html>";
        assert_eq!(
            numero_do_historico(cabecalhos),
            Err(ErroHistorico::Html(ErroHtml::repetido(
                "cabecalhos_html_do_historico_repetidos"
            )))
        );
        assert_eq!(
            ler_historico(cabecalhos, "000123"),
            Err(ErroHistorico::Html(ErroHtml::repetido(
                "cabecalhos_html_do_historico_repetidos"
            )))
        );
    }
}
