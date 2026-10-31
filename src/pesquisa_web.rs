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
    LinkDeProcessoSemClasseReconhecida,
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
            Self::LinkDeProcessoSemClasseReconhecida => {
                "link_de_processo_sem_classe_reconhecida"
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
    // A classe CSS do link não é garantia de que ele seja o único link
    // de trabalho na tabela. Uma versão diferente do SEI pode perder
    // essa classe e ocultar processos na projeção: falhar fechado.
    let links_de_trabalho = tabela.query_selector(parser, "a").map_or(0, |links| {
        links
            .filter(|no| {
                no.get(parser)
                    .and_then(tl::Node::as_tag)
                    .and_then(|tag| tag.attributes().get("href").flatten())
                    .is_some_and(|href| {
                        href.as_utf8_str()
                            .strip_prefix("controlador.php?")
                            .is_some_and(|query| {
                                query.split('&').enumerate().any(|(indice, campo)| {
                                    campo == "acao=procedimento_trabalhar"
                                        || (indice > 0
                                            && campo == "amp;acao=procedimento_trabalhar")
                                })
                            })
                    })
            })
            .count()
    });
    if links_de_trabalho != resultados.len() {
        return Err(ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida);
    }
    Ok(resultados)
}

#[cfg(test)]
mod tests;
