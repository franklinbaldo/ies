//! Total do relatório de acervo e link de detalhamento.
use super::{ANDAMENTO_ABERTO, ErroAcervo, HtmlServido, arvore, texto};
use crate::legenda::ler_legenda;

/// Seleciona o TOTAL da tabela 4, não um tipo ou outra estatística.
/// Resolve o link observado com a guarda comum dos endereços assinados.
pub(super) fn total_do_resultado(
    html: &HtmlServido,
    instalacao: &crate::instalacao::Instalacao,
) -> Result<(usize, Option<String>), ErroAcervo> {
    use crate::acao_do_sei::AcaoDoSei;
    use crate::arvore_processo::endereco::conferir_endereco_relativo_por;
    use crate::consulta_do_controlador::ConsultaDoControlador;
    let dom = arvore(html);
    let mut tabelas = dom.elementos("table").filter(|t| {
        t.elementos("caption")
            .any(|c| texto(c).trim_end_matches(':') == ANDAMENTO_ABERTO)
    });
    let tabela = tabelas.next().ok_or(ErroAcervo::ResultadoSemTotal)?;
    if tabelas.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let mut linhas = tabela.elementos("tr").filter(|tr| {
        tr.atributo("class")
            .is_some_and(|c| c.split_whitespace().any(|v| v == "totalEstatisticas"))
    });
    let linha = linhas.next().ok_or(ErroAcervo::ResultadoSemTotal)?;
    if linhas.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let mut links = linha.elementos("a").filter_map(|a| {
        let (_, resto) = a.atributo("onclick")?.split_once("abrirDetalhe('")?;
        let (url, _) = resto.split_once('\'')?;
        Some((texto(a), url.replace("&amp;", "&")))
    });
    let Some((numero, url)) = links.next() else {
        // Hipótese mínima para acervo vazio: TOTAL explicitamente zero,
        // sem link de detalhe. Ausência da tabela/linha nunca é zero.
        let celulas: Vec<_> = linha.filhos().filter(|c| c.nome() == "td").map(texto).collect();
        if linha.elementos("a").next().is_none()
            && celulas.len() == 2
            && celulas[0] == "TOTAL:"
            && celulas[1] == "0"
        {
            return Ok((0, None));
        }
        return Err(ErroAcervo::ResultadoSemTotal);
    };
    if links.next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let (_, query) = url.split_once('?').ok_or(ErroAcervo::ResultadoSemTotal)?;
    let consulta = ConsultaDoControlador::ler(query).map_err(|_| ErroAcervo::ResultadoSemTotal)?;
    if consulta.valores("id_tipo_procedimento").next().is_some() {
        return Err(ErroAcervo::ResultadoSemTotal);
    }
    let url = conferir_endereco_relativo_por(
        instalacao,
        &url,
        AcaoDoSei::EstatisticasDetalharUnidade,
        "tabela_estatisticas",
        "4",
    )
    .map_err(|_| ErroAcervo::ResultadoSemTotal)?;
    let total = ler_legenda(&format!("{numero} registros:"))?.total;
    Ok((total, Some(url)))
}
