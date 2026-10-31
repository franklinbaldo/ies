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
    LinkDeProcessoForaDaLinhaDePesquisa,
    LinkDePesquisaNaoAbreProcesso,
    LinkDePesquisaSemIdentificadorDoProcesso,
    LinkDePesquisaComIdentificadoresRepetidos,
    LinkDePesquisaComChaveNaoCanonica,
    LinkDePesquisaComAcaoSemIgual,
    LinkDePesquisaComIdentificadorSemIgual,
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
            Self::LinkDeProcessoForaDaLinhaDePesquisa => {
                "link_de_processo_fora_da_linha_de_pesquisa"
            }
            Self::LinkDePesquisaNaoAbreProcesso => "link_de_pesquisa_nao_abre_processo",
            Self::LinkDePesquisaSemIdentificadorDoProcesso => {
                "link_de_pesquisa_sem_identificador_do_processo"
            }
            Self::LinkDePesquisaComIdentificadoresRepetidos => {
                "link_de_pesquisa_com_identificadores_repetidos"
            }
            Self::LinkDePesquisaComChaveNaoCanonica => "link_de_pesquisa_com_chave_nao_canonica",
            Self::LinkDePesquisaComAcaoSemIgual => "link_de_pesquisa_com_acao_sem_igual",
            Self::LinkDePesquisaComIdentificadorSemIgual => {
                "link_de_pesquisa_com_identificador_sem_igual"
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

// O controlador PHP decodifica nomes de query e normaliza pontos/espacos
// para '_'. %5F, '.' ou '+' podem sobrescrever acao/id_procedimento.
// Os nomes devem ser literais ASCII, inclusive os parametros auxiliares.
fn validar_nomes_dos_parametros(campos: &[&str]) -> Result<(), ErroResultadoPesquisa> {
    for campo in campos {
        // Para o controlador PHP, `acao` e `id_procedimento` sem `=`
        // são campos vazios e podem substituir os valores anteriores.
        // Não deixar uma chave obrigatória sem valor parecer parâmetro auxiliar.
        let chave = if let Some((nome, _)) = campo.split_once('=') {
            nome
        } else {
            match *campo {
                "acao" => return Err(ErroResultadoPesquisa::LinkDePesquisaComAcaoSemIgual),
                "id_procedimento" => {
                    return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorSemIgual);
                }
                _ => campo,
            }
        };
        if chave.is_empty()
            || !chave
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(ErroResultadoPesquisa::LinkDePesquisaComChaveNaoCanonica);
        }
    }
    Ok(())
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
        // Somente após um separador & o prefixo amp; pode ser sobra de
        // &amp; no HTML. No primeiro campo, amp;acao é outra chave, não acao.
        let campos: Vec<_> = query
            .split('&')
            .enumerate()
            .map(|(indice, campo)| {
                if indice == 0 {
                    campo
                } else {
                    campo.strip_prefix("amp;").unwrap_or(campo)
                }
            })
            .collect();
        let acoes: Vec<_> = campos
            .iter()
            .filter_map(|campo| campo.strip_prefix("acao="))
            .collect();
        if acoes != ["procedimento_trabalhar"] {
            return Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso);
        }
        validar_nomes_dos_parametros(&campos)?;
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
        // O identificador interno é decimal: 7 e 007 denotam a mesma chave.
        // Normalização por string evita overflow em IDs maiores que u64.
        if !ids_observados.insert(primeiro_id.trim_start_matches('0').to_owned()) {
            return Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa);
        }
        resultados.push(ResultadoPesquisa { numero });
    }
    // A seleção por classe das linhas pode omitir links reais se a versão
    // do SEI mudar o HTML. Nunca devolver subconjunto silencioso como se
    // representasse todos os links observáveis da tabela de resultados.
    let links_da_tabela = tabela
        .query_selector(parser, "a.protocoloNormal")
        .map_or(0, Iterator::count);
    if links_da_tabela != resultados.len() {
        return Err(ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa);
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
    fn linha_sem_classe_nao_pode_ocultar_processo_da_contagem() {
        // A tabela continua contendo um link SEI de processo legítimo, mas
        // sua linha perdeu a classe de resultado: ignorá-la perde um NUP.
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa)
        );
    }

    #[test]
    fn chave_amp_acao_no_primeiro_parametro_nao_e_acao_valida() {
        let html = RESULTADO.replace(
            "?acao=procedimento_trabalhar",
            "?amp;acao=procedimento_trabalhar",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
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
    fn chaves_que_o_php_pode_normalizar_nao_alteram_identidade_silenciosamente() {
        // PHP converte pontos e espacos em '_' e decodifica percent/+
        // antes de escolher o valor do parametro; o parser nao pode
        // aceitar somente a chave literal e ignorar seu alias.
        for extra in [
            "id.procedimento=8",
            "id+procedimento=8",
            "id%5Fprocedimento=8",
            "a%63ao=documento_imprimir_web",
        ] {
            let html = RESULTADO.replace(
                "&amp;id_procedimento=7",
                &format!("&amp;id_procedimento=7&amp;{extra}"),
            );
            assert_eq!(
                ler_resultados(&html),
                Err(ErroResultadoPesquisa::LinkDePesquisaComChaveNaoCanonica),
                "chave alternativa nao pode mudar a acao ou o ID: {extra}"
            );
        }
    }

    #[test]
    fn parametro_acao_sem_igual_nao_pode_sobrescrever_acao_valida() {
        // O parse de query do PHP trata `acao` sem "=" como valor vazio.
        // Um valor vazio posterior sobrepõe a ação reconhecida pelo parser.
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;acao",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComAcaoSemIgual)
        );
    }

    #[test]
    fn parametro_id_sem_igual_nao_pode_sobrescrever_identificador_valido() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;id_procedimento",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorSemIgual)
        );
    }

    #[test]
    fn parametros_adicionais_com_nomes_canonicos_preservam_resultado() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;infra_hash=exemplo&amp;id-contexto=8",
        );
        assert_eq!(ler_resultados(&html).unwrap().len(), 1);
    }

    #[test]
    fn tabela_sem_processos_observados_nao_inventa_identidade() {
        assert_eq!(
            ler_resultados("<table class='pesquisaResultado'></table>"),
            Ok(vec![])
        );
    }
}
