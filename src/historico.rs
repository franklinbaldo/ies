//! Leitura estritamente offline do histórico Web de um processo.
//! Não confundir histórico parcial com a representação integral do processo.
use crate::processo::{Andamento, DetalheProcesso, NumeroProcesso};

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

pub fn numero_do_historico(html: &str) -> Result<String, &'static str> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| "html_invalido")?;
    let parser = dom.parser();
    let titulo = dom.query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or("titulo_ausente")?;
    let (_, depois) = titulo.split_once("Histórico do Processo ").ok_or("titulo_invalido")?;
    let candidatos: Vec<_> = depois.split_whitespace()
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit() || b == b'.' || b == b'/' || b == b'-'))
        .collect();
    if candidatos.len() != 1 { return Err("numero_ambiguo"); }
    Ok(candidatos[0].to_owned())
}

pub fn ler_historico(html: &str, numero: &str) -> Result<DetalheProcesso, &'static str> {
    let numero = NumeroProcesso::novo(numero).map_err(|_| "numero_invalido")?;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| "html_invalido")?;
    let parser = dom.parser();
    let titulo = dom
        .query_selector("title")
        .and_then(|mut nodes| nodes.next())
        .and_then(|no| no.get(parser))
        .map(|no| texto(no, parser))
        .ok_or("titulo_ausente")?;
    // Não converter um histórico de outro processo numa resposta para o NUP pedido.
    if !titulo
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|item| item == numero.as_str())
    {
        return Err("processo_divergente");
    }
    let tabela = dom
        .get_element_by_id("tblHistorico")
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or("historico_ausente")?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut nodes| nodes.next().is_some())
    {
        return Err("historico_invalido");
    }
    let mut detalhe = DetalheProcesso::novo(numero);
    let linhas = tabela
        .query_selector(parser, "tr")
        .ok_or("historico_invalido")?;
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or("historico_invalido")?;
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
            return Err("linha_invalida");
        }
        detalhe.andamentos.push(Andamento {
            ocorrido_em: Some(cells[0].clone()),
            unidade: (!cells[1].is_empty()).then(|| cells[1].clone()),
            usuario: (!cells[2].is_empty()).then(|| cells[2].clone()),
            descricao: cells[3].clone(),
        });
    }
    if detalhe.andamentos.is_empty() {
        return Err("historico_vazio");
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
            "processo_divergente"
        );
        let invalido = "<title>Histórico do Processo 202600005002487</title>        <table id='tblHistorico'><tr><td>12/07/2026</td></tr></table>";
        assert_eq!(
            ler_historico(invalido, "202600005002487").unwrap_err(),
            "linha_invalida"
        );
    }
}
