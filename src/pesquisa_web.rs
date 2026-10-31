//! Projeção tipada de processos nas respostas HTML de pesquisa do SEI Web.
//! Parser puro: não executa links nem presume permissão/resultado live.
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoPesquisa {
    pub numero: NumeroProcesso,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroResultadoPesquisa {
    HtmlDePesquisaNaoInterpretado,
    TabelaDePesquisaAusente,
    LinhaDePesquisaSemLinkDoProcesso,
    LinkDePesquisaNaoAbreProcesso,
    NumeroDePesquisa(NumeroProcessoInvalido),
    NumeroDePesquisaRepetido,
}

impl ErroResultadoPesquisa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDePesquisaNaoInterpretado => "html_de_pesquisa_nao_interpretado",
            Self::TabelaDePesquisaAusente => "tabela_de_pesquisa_ausente",
            Self::LinhaDePesquisaSemLinkDoProcesso => "linha_de_pesquisa_sem_link_do_processo",
            Self::LinkDePesquisaNaoAbreProcesso => "link_de_pesquisa_nao_abre_processo",
            Self::NumeroDePesquisa(erro) => erro.subcodigo(),
            Self::NumeroDePesquisaRepetido => "numero_de_pesquisa_repetido",
        }
    }
}

/// Resultados observados somente na tabela de pesquisa retornada pelo SEI.
/// Nunca serializa links assinados, IDs técnicos, trechos HTML ou filtros.
pub fn ler_resultados(html: &str) -> Result<Vec<ResultadoPesquisa>, ErroResultadoPesquisa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado)?;
    let parser = dom.parser();
    let tabela = dom
        .query_selector("table.pesquisaResultado")
        .and_then(|mut itens| itens.next())
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let linhas = tabela
        .query_selector(parser, "tr.pesquisaTituloRegistro")
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let mut observados = HashSet::new();
    let mut resultados = Vec::new();
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let link = tag
            .query_selector(parser, "a.protocoloNormal")
            .and_then(|mut itens| itens.next())
            .and_then(|no| no.get(parser))
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let href = link
            .attributes()
            .get("href")
            .flatten()
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?
            .as_utf8_str();
        let acao_confirmada = href.starts_with("controlador.php?")
            && href
                .split(['?', '&'])
                .any(|campo| campo == "acao=procedimento_trabalhar");
        if !acao_confirmada {
            return Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso);
        }
        let numero = NumeroProcesso::novo(&link.inner_text(parser))
            .map_err(ErroResultadoPesquisa::NumeroDePesquisa)?;
        if !observados.insert(numero.as_str().to_owned()) {
            return Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido);
        }
        resultados.push(ResultadoPesquisa { numero });
    }
    Ok(resultados)
}

#[cfg(test)]
mod tests {
    use super::*;
    const RESULTADO: &str = r#"<html><table class="pesquisaResultado">
      <tr class="pesquisaTituloRegistro"><td class="pesquisaTituloEsquerda">
      <a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00001.123456/2026-07</a>
      </td></tr></table></html>"#;

    #[test]
    fn numero_encontrado_na_pesquisa_preserva_zeros_e_nao_link_assinado() {
        let encontrados = ler_resultados(RESULTADO).unwrap();
        assert_eq!(encontrados.len(), 1);
        assert_eq!(encontrados[0].numero.as_str(), "00001.123456/2026-07");
        let saida = serde_json::to_string(&encontrados).unwrap();
        assert!(!saida.contains("id_procedimento"));
        assert!(!saida.contains("controlador.php"));
    }

    #[test]
    fn nao_converte_formulario_vazio_em_resultado_de_busca() {
        assert_eq!(
            ler_resultados("<form id='frmPesquisaProtocolo'></form>"),
            Err(ErroResultadoPesquisa::TabelaDePesquisaAusente)
        );
    }

    #[test]
    fn link_para_outra_acao_nao_vira_processo() {
        let html = RESULTADO.replace("acao=procedimento_trabalhar", "acao=documento_imprimir_web");
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
        );
    }

    #[test]
    fn numeros_repetidos_da_mesma_resposta_sao_recusados() {
        let html = RESULTADO.replace("</table>", r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar">00001.123456/2026-07</a></td></tr></table>"#);
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido)
        );
    }

    #[test]
    fn tabela_sem_processos_observados_nao_inventa_identidade() {
        assert_eq!(
            ler_resultados("<table class='pesquisaResultado'></table>"),
            Ok(vec![])
        );
    }
}
