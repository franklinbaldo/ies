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
    TabelasDePesquisaRepetidas,
    LinhaDePesquisaSemLinkDoProcesso,
    LinhaDePesquisaComMaisDeUmLinkDoProcesso,
    LinkDePesquisaNaoAbreProcesso,
    LinkDePesquisaSemIdentificadorDoProcesso,
    LinkDePesquisaComIdentificadoresRepetidos,
    LinkDePesquisaComIdentificadorDoProcessoNaoDecimal,
    IdentificadorDoProcessoRepetidoNaPesquisa,
    NumeroDePesquisa(NumeroProcessoInvalido),
    NumeroDePesquisaRepetido,
}

impl ErroResultadoPesquisa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDePesquisaNaoInterpretado => "html_de_pesquisa_nao_interpretado",
            Self::TabelaDePesquisaAusente => "tabela_de_pesquisa_ausente",
            Self::TabelasDePesquisaRepetidas => "tabelas_de_pesquisa_repetidas",
            Self::LinhaDePesquisaSemLinkDoProcesso => "linha_de_pesquisa_sem_link_do_processo",
            Self::LinhaDePesquisaComMaisDeUmLinkDoProcesso => {
                "linha_de_pesquisa_com_mais_de_um_link_do_processo"
            }
            Self::LinkDePesquisaNaoAbreProcesso => "link_de_pesquisa_nao_abre_processo",
            Self::LinkDePesquisaSemIdentificadorDoProcesso => {
                "link_de_pesquisa_sem_identificador_do_processo"
            }
            Self::LinkDePesquisaComIdentificadoresRepetidos => {
                "link_de_pesquisa_com_identificadores_repetidos"
            }
            Self::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal => {
                "link_de_pesquisa_com_identificador_do_processo_nao_decimal"
            }
            Self::IdentificadorDoProcessoRepetidoNaPesquisa => {
                "identificador_do_processo_repetido_na_pesquisa"
            }
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
    let mut tabelas = dom
        .query_selector("table.pesquisaResultado")
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let primeira = tabelas
        .next()
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    if tabelas.next().is_some() {
        return Err(ErroResultadoPesquisa::TabelasDePesquisaRepetidas);
    }
    let tabela = primeira
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let linhas = tabela
        .query_selector(parser, "tr.pesquisaTituloRegistro")
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let mut observados = HashSet::new();
    let mut ids_observados = HashSet::new();
    let mut resultados = Vec::new();
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let mut links = tag
            .query_selector(parser, "a.protocoloNormal")
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let link_no = links
            .next()
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        if links.next().is_some() {
            return Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso);
        }
        let link = link_no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let href = link
            .attributes()
            .get("href")
            .flatten()
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?
            .as_utf8_str();
        // Não aceitar múltiplos parâmetros de ação: proxies/servidores podem
        // escolher o primeiro ou o último, tornando o destino indeterminado.
        // tl preserva entidades HTML em atributos de algumas versões do SEI.
        let query = href
            .strip_prefix("controlador.php?")
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?;
        let campos: Vec<_> = query
            .split('&')
            .map(|campo| campo.strip_prefix("amp;").unwrap_or(campo))
            .collect();
        let acoes: Vec<_> = campos
            .iter()
            .filter_map(|campo| campo.strip_prefix("acao="))
            .collect();
        if acoes != ["procedimento_trabalhar"] {
            return Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso);
        }
        // Um link de trabalho sem ID interno não endereça um processo.
        // Dois IDs tornam o destino dependente da interpretação do servidor.
        // O identificador é validado somente para uso local e não é serializado.
        let mut ids = campos
            .iter()
            .filter_map(|campo| campo.strip_prefix("id_procedimento="));
        let primeiro_id = ids
            .next()
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso)?;
        if ids.next().is_some() {
            return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadoresRepetidos);
        }
        if primeiro_id.is_empty() {
            return Err(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso);
        }
        if !primeiro_id.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal);
        }
        let numero = NumeroProcesso::novo(&link.inner_text(parser))
            .map_err(ErroResultadoPesquisa::NumeroDePesquisa)?;
        if !observados.insert(numero.as_str().to_owned()) {
            return Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido);
        }
        if !ids_observados.insert(primeiro_id.to_owned()) {
            return Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa);
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
        let html = RESULTADO.replace("</table>", r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00001.123456/2026-07</a></td></tr></table>"#);
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido)
        );
    }

    #[test]
    fn recusa_duas_acoes_no_mesmo_link_antes_de_projetar_nup() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;acao=documento_imprimir_web&amp;id_procedimento=7",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
        );
    }

    #[test]
    fn duas_tabelas_de_pesquisa_nao_podem_ocultar_resultados() {
        let html = format!(
            "{RESULTADO}<table class='pesquisaResultado'><tr class='pesquisaTituloRegistro'><td><a class='protocoloNormal' href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8'>00002.123456/2026-07</a></td></tr></table>"
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::TabelasDePesquisaRepetidas)
        );
    }

    #[test]
    fn linha_com_dois_links_de_processos_nao_escolhe_o_primeiro_silenciosamente() {
        let html = RESULTADO.replace(
            "</a>",
            r#"</a><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00002.123456/2026-07</a>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso)
        );
    }

    #[test]
    fn link_sem_identificador_interno_nao_e_um_link_de_processo() {
        let html = RESULTADO.replace("&amp;id_procedimento=7", "");
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso)
        );
    }

    #[test]
    fn link_com_identificadores_repetidos_nao_escolhe_destino_arbitrario() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;id_procedimento=8",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadoresRepetidos)
        );
    }

    #[test]
    fn link_com_id_interno_nao_decimal_nao_vira_processo_observado() {
        for id in ["abc", "-7", "%37"] {
            let html = RESULTADO.replace(
                "&amp;id_procedimento=7",
                &format!("&amp;id_procedimento={id}"),
            );
            assert_eq!(
                ler_resultados(&html),
                Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal),
                "identificador interno {id} deve ser decimal ASCII"
            );
        }
    }

    #[test]
    fn dois_nups_distintos_nao_podem_apontar_para_o_mesmo_processo_interno() {
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa)
        );
    }

    #[test]
    fn id_interno_com_zeros_a_esquerda_nao_pode_representar_dois_processos() {
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=007">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa),
            "o mesmo ID numérico (7 e 007) não pode identificar NUPs diferentes"
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
