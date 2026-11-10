//! Leitor estrutural da lista mensal de retornos (somente leitura).
//! Fonte dos cenarios: Todos `tests/test_unidade_listar_retornos_010.py` @323eb01.
//! Nao interpreta JavaScript, nao escolhe linhas fora de tabela reconhecida.
use crate::html_atributos::valor_do_atributo;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Retorno {
    pub(crate) situacao: &'static str,
    pub(crate) numero: String,
    pub(crate) unidade: String,
    pub(crate) data_programada: String,
    #[serde(rename = "data_retorno")]
    pub(crate) retornado_em: Option<String>,
    pub(crate) prazo_restante: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRetornos {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    TelaDeRetornosDesconhecida,
    ListaDeRetornosIncompleta,
    PaginacaoDosRetornosNaoObservada,
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroRetornos> {
    valor_do_atributo(tag, nome).map_err(ErroRetornos::Atributo)
}

fn texto(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&tag.inner_text(parser))
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn identificador(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn situacao_da_tabela(summary: &str) -> Option<&'static str> {
    let nome = identificador(summary);
    if nome.contains("aguardando retorno") {
        Some("aguardando")
    } else if nome.contains("para devolver") {
        Some("devolver")
    } else {
        None
    }
}

pub(crate) fn data_valida(s: &str) -> bool {
    let partes: Vec<_> = s.split('/').collect();
    let [dia, mes, ano] = partes.as_slice() else {
        return false;
    };
    if dia.len() != 2
        || mes.len() != 2
        || ano.len() != 4
        || !s.bytes().all(|b| b.is_ascii_digit() || b == b'/')
    {
        return false;
    }
    let (Ok(d), Ok(m), Ok(a)) = (dia.parse::<u8>(), mes.parse::<u8>(), ano.parse::<u16>()) else {
        return false;
    };
    if a == 0 || !(1..=12).contains(&m) {
        return false;
    }
    let bissexto = a.is_multiple_of(4) && (!a.is_multiple_of(100) || a.is_multiple_of(400));
    let dias = match m {
        2 if bissexto => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    d >= 1 && d <= dias
}

pub(crate) fn ultimo_dia(ano: u16, mes: u8) -> Option<u8> {
    if ano == 0 || !(1..=12).contains(&mes) {
        return None;
    }
    let bissexto = ano.is_multiple_of(4) && (!ano.is_multiple_of(100) || ano.is_multiple_of(400));
    Some(match mes {
        2 if bissexto => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    })
}

fn ler_tabela(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    situacao: &'static str,
) -> Result<Vec<Retorno>, ErroRetornos> {
    let legendas: Vec<_> = tabela
        .query_selector(parser, "caption")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroRetornos::TelaDeRetornosDesconhecida);
    };
    let declarado = crate::caixa::total_da_legenda(&texto(legenda, parser))
        .ok_or(ErroRetornos::TelaDeRetornosDesconhecida)?;
    let titulos: Vec<String> = tabela
        .query_selector(parser, "th")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .map(|t| identificador(&texto(t, parser)))
        .collect();
    let indice = |nome: &str| {
        titulos
            .iter()
            .position(|c| c == nome)
            .ok_or(ErroRetornos::TelaDeRetornosDesconhecida)
    };
    let (processo, unidade, data) = (
        indice("processo")?,
        indice("unidade")?,
        indice("data programada")?,
    );
    if titulos.iter().collect::<HashSet<_>>().len() != titulos.len() {
        return Err(ErroRetornos::TelaDeRetornosDesconhecida);
    }
    let retorno = titulos.iter().position(|c| c == "data de retorno efetiva");
    let prazo = titulos.iter().position(|c| c == "prazo restante");
    let mut lista = Vec::new();
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let Some(linha) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let celulas: Vec<_> = linha
            .query_selector(parser, "td")
            .into_iter()
            .flatten()
            .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
            .collect();
        if celulas.is_empty() {
            continue;
        }
        if celulas.len() != titulos.len() {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        let valor = |pos: usize| texto(celulas[pos], parser);
        let numero = valor(processo);
        let origem = valor(unidade);
        let programada = valor(data);
        if numero.is_empty() || origem.is_empty() || !data_valida(&programada) {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        let opcional = |pos: Option<usize>| {
            pos.and_then(|p| {
                let v = valor(p);
                (!v.is_empty()).then_some(v)
            })
        };
        lista.push(Retorno {
            situacao,
            numero,
            unidade: origem,
            data_programada: programada,
            retornado_em: opcional(retorno),
            prazo_restante: opcional(prazo),
        });
    }
    if lista.len() != declarado {
        return Err(ErroRetornos::ListaDeRetornosIncompleta);
    }
    Ok(lista)
}

/// So retorna uma lista quando cada situacao tem tabela conferida ou
/// evidencia explicita da area sem registros.
pub(crate) fn extrair_retornos(html: &str) -> Result<Vec<Retorno>, ErroRetornos> {
    for prefixo in ["Aguardando", "Devolver"] {
        if crate::paginacao_controle::paginador_com_prefixo(html, prefixo)
            .map_err(|_| ErroRetornos::TelaDeRetornosDesconhecida)?
        {
            return Err(ErroRetornos::PaginacaoDosRetornosNaoObservada);
        }
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroRetornos::TelaDeRetornosDesconhecida)?;
    let parser = dom.parser();
    let mut situacoes = HashSet::new();
    let mut lista = Vec::new();
    for no in dom.query_selector("table").into_iter().flatten() {
        let Some(tabela) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(summary) = atributo(tabela, "summary")? else {
            continue;
        };
        let Some(situacao) = situacao_da_tabela(&summary) else {
            continue;
        };
        if !situacoes.insert(situacao)
            || tabela
                .query_selector(parser, "table")
                .is_some_and(|mut it| it.next().is_some())
        {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        lista.extend(ler_tabela(tabela, parser, situacao)?);
    }
    // A lista mensal completa observada em todas as sete instalacoes pode
    // omitir "Para Devolver", mas so dentro da area mensal reconhecida.
    // Tabela solta sem essa moldura nao comprova completude.
    let mut vazios = HashSet::new();
    let mut areas_aguardando = 0usize;
    let mut areas_observadas = HashSet::new();
    let mut area_aguardando_com_tabela = false;
    for no in dom.query_selector("div").into_iter().flatten() {
        let Some(area) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let situacao = match atributo(area, "id")?.as_deref() {
            Some("divAguardandoAreaTabela") => "aguardando",
            Some("divDevolverAreaTabela") => "devolver",
            _ => continue,
        };
        // Duas areas com o mesmo id nao permitem provar a completude,
        // mesmo quando uma delas afirma que o mes esta vazio.
        if !areas_observadas.insert(situacao) {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        if situacao == "aguardando" {
            areas_aguardando += 1;
            let mut reconhecidas = Vec::new();
            for no in area.query_selector(parser, "table").into_iter().flatten() {
                let Some(tabela) = no.get(parser).and_then(tl::Node::as_tag) else {
                    continue;
                };
                if let Some(summary) = atributo(tabela, "summary")?
                    && let Some(tipo) = situacao_da_tabela(&summary)
                {
                    reconhecidas.push(tipo);
                }
            }
            area_aguardando_com_tabela = reconhecidas == ["aguardando"];
        }
        if identificador(&texto(area, parser)) == "nenhum registro encontrado."
            && !vazios.insert(situacao)
        {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
    }
    if !situacoes.is_empty() {
        let duas_situacoes = ["aguardando", "devolver"]
            .iter()
            .all(|situacao| situacoes.contains(situacao) || vazios.contains(situacao));
        let area_mensal_observada = areas_aguardando == 1
            && area_aguardando_com_tabela
            && situacoes.len() == 1
            && situacoes.contains("aguardando");
        if duas_situacoes || area_mensal_observada {
            Ok(lista)
        } else {
            Err(ErroRetornos::ListaDeRetornosIncompleta)
        }
    } else if vazios.contains("aguardando") {
        Ok(Vec::new())
    } else {
        Err(ErroRetornos::TelaDeRetornosDesconhecida)
    }
}

#[cfg(test)]
#[path = "unidade_retornos/acervo.rs"]
mod acervo;

#[cfg(test)]
mod tests {
    use super::*;
    fn tabela(situacao: &str, numero: &str, unidade: &str) -> String {
        format!(
            "<table summary='{situacao}'><caption>1 registro:</caption><thead><tr><th>Processo</th><th>Unidade</th><th>Data Programada</th></tr></thead><tbody><tr><td><a>{numero}</a></td><td>{unidade}</td><td>15/10/2026</td></tr></tbody></table>"
        )
    }
    #[test]
    fn mes_com_duas_situacoes_preserva_linhas_e_ordem() {
        // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
        let html = format!(
            "{}{}",
            tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1"),
            tabela("Para Devolver", "NUP-B", "U2")
        );
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 2);
        assert_eq!(itens[0].situacao, "aguardando");
        assert_eq!(itens[0].numero, "NUP-A");
        assert_eq!(itens[1].situacao, "devolver");
        assert_eq!(itens[1].unidade, "U2");
    }
    #[test]
    fn mes_vazio_exige_aviso_estrutural() {
        // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
        let html = "<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>";
        assert_eq!(extrair_retornos(html).unwrap(), Vec::<Retorno>::new());
        assert_eq!(
            extrair_retornos("<div>outra tela</div>"),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn legenda_maior_que_as_linhas_nao_publica_resultado_parcial() {
        let html = tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
            .replace("1 registro:", "2 registros:");
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::ListaDeRetornosIncompleta)
        );
    }
    #[test]
    fn tabelas_duplicadas_sao_recusadas() {
        let t = tabela("Para Devolver", "NUP-A", "U1");
        assert_eq!(
            extrair_retornos(&format!("{t}{t}")),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn pagina_apresenta_paginador_sem_replay_falha_fechado() {
        let html = tabela("Para Devolver", "NUP-A", "U1")
            + "<a id='lnkDevolverProximaPaginaSuperior'>proxima</a>";
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::PaginacaoDosRetornosNaoObservada)
        );
    }
    #[test]
    fn tabela_unica_sem_evidencia_da_outra_situacao_e_incompleta() {
        // Na UnB, as duas situacoes tem secoes independentes; nao publicar metade do mes.
        let html = tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1");
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::ListaDeRetornosIncompleta)
        );
    }
    #[test]
    fn area_mensal_observada_com_aguardando_nao_inventa_devolver() {
        // Replay UnB/GO/PEN3: o mes completo pode omitir a tabela Para Devolver.
        // Uma tabela solta sem a area mensal continua recusada.
        let html = format!(
            "<div id='divAguardandoAreaTabela'>{}<table><tr><td>layout</td></tr></table></div>",
            tabela(
                "Tabela de processos aguardando retorno de outras unidades",
                "NUP-A",
                "U1"
            )
        );
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].situacao, "aguardando");
    }
    #[test]
    fn tabela_unica_com_outra_area_explicitamente_vazia_pode_ser_lida() {
        // A ausencia da outra tabela so vale se a sua area disser que nao ha registros.
        let html = tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
            + "<div id='divDevolverAreaTabela'>Nenhum registro encontrado.</div>";
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].situacao, "aguardando");
    }
    #[test]
    fn area_aguardando_duplicada_nao_parece_mes_vazio_completo() {
        // Estrutura sintética: area repetida torna ambigua a completude do mes.
        let html = "<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>                    <div id='divAguardandoAreaTabela'>Area pendente de carregamento</div>";
        assert_eq!(
            extrair_retornos(html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn area_devolver_duplicada_nao_parece_mes_completo() {
        // O aviso de vazio de uma area nao prova a integridade da outra duplicata.
        let html = format!(
            "{}<div id='divDevolverAreaTabela'>Nenhum registro encontrado.</div>             <div id='divDevolverAreaTabela'>Area pendente de carregamento</div>",
            tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
        );
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn data_fora_do_calendario_nao_e_publicada() {
        let html = tabela("Para Devolver", "NUP-A", "U1").replace("15/10/2026", "31/02/2026");
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
        assert!(data_valida("29/02/2024"));
        assert!(!data_valida("29/02/2025"));
    }
}
