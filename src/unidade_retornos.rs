//! Leitor estrutural da lista mensal de retornos (somente leitura).
//! Fonte dos cenarios: Todos `tests/test_unidade_listar_retornos_010.py` @323eb01.
//! Nao interpreta JavaScript, nao escolhe linhas fora de tabela reconhecida.
use serde::{Serialize, Serializer};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Retorno {
    pub(crate) situacao: &'static str,
    pub(crate) numero: String,
    pub(crate) unidade: String,
    pub(crate) data_programada: String,
    #[serde(rename = "data_retorno")]
    pub(crate) retornado_em: DataDeRetorno,
    pub(crate) prazo_restante: PrazoRestante,
}

/// Célula "Data de Retorno Efetiva": vazia enquanto a unidade não devolveu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DataDeRetorno {
    Pendente,
    Em(String),
}

impl Serialize for DataDeRetorno {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pendente => s.serialize_str("pendente"),
            Self::Em(data) => s.serialize_str(data),
        }
    }
}

/// Célula "Prazo Restante": vazia quando a tabela não publica prazo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrazoRestante {
    SemPrazo,
    Em(String),
}

impl Serialize for PrazoRestante {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::SemPrazo => s.serialize_str("sem_prazo"),
            Self::Em(prazo) => s.serialize_str(prazo),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRetornos {
    TelaDeRetornosDesconhecida,
    Legenda(#[from] crate::legenda::ErroLegenda),
    ListaDeRetornosIncompleta,
    ListaDeRetornosComMaisDeUmaPagina,
    /// A tabela não tem a coluna "Data de Retorno Efetiva".
    ColunaDataDeRetornoAusente,
    /// A tabela não tem a coluna "Prazo Restante".
    ColunaPrazoRestanteAusente,
}

fn texto(tag: crate::tela_html::Elemento<'_>) -> String {
    tag.texto().split_whitespace().collect::<Vec<_>>().join(" ")
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

mod data;
pub(crate) use data::{DataCivil, MesAno};

pub(crate) fn data_valida(s: &str) -> bool {
    DataCivil::ler(s).is_some()
}

fn ler_tabela(
    tabela: crate::tela_html::Elemento<'_>,
    situacao: &'static str,
) -> Result<Vec<Retorno>, ErroRetornos> {
    let legendas: Vec<_> = tabela.elementos("caption").collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroRetornos::TelaDeRetornosDesconhecida);
    };
    let declarado = crate::legenda::ler_legenda(&texto(*legenda))?.total;
    let titulos: Vec<String> = tabela
        .elementos("th")
        .map(|t| identificador(&texto(t)))
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
    let retorno = titulos
        .iter()
        .position(|c| c == "data de retorno efetiva")
        .ok_or(ErroRetornos::ColunaDataDeRetornoAusente)?;
    let prazo = titulos
        .iter()
        .position(|c| c == "prazo restante")
        .ok_or(ErroRetornos::ColunaPrazoRestanteAusente)?;
    let mut lista = Vec::new();
    for no in tabela.elementos("tr") {
        let celulas: Vec<_> = no.elementos("td").collect();
        if celulas.is_empty() {
            continue;
        }
        if celulas.len() != titulos.len() {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        let valor = |pos: usize| {
            celulas
                .get(pos)
                .copied()
                .map(texto)
                .ok_or(ErroRetornos::TelaDeRetornosDesconhecida)
        };
        let numero = valor(processo)?;
        let origem = valor(unidade)?;
        let programada = valor(data)?;
        if numero.is_empty() || origem.is_empty() || !data_valida(&programada) {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        let devolucao = valor(retorno)?;
        let prazo_lido = valor(prazo)?;
        lista.push(Retorno {
            situacao,
            numero,
            unidade: origem,
            data_programada: programada,
            retornado_em: if devolucao.is_empty() {
                DataDeRetorno::Pendente
            } else {
                DataDeRetorno::Em(devolucao)
            },
            prazo_restante: if prazo_lido.is_empty() {
                PrazoRestante::SemPrazo
            } else {
                PrazoRestante::Em(prazo_lido)
            },
        });
    }
    if lista.len() != declarado {
        return Err(ErroRetornos::ListaDeRetornosIncompleta);
    }
    Ok(lista)
}

/// So retorna uma lista quando cada situacao tem tabela conferida ou
/// evidencia explicita da area sem registros.
pub(crate) fn extrair_retornos(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<Retorno>, ErroRetornos> {
    for prefixo in ["Aguardando", "Devolver"] {
        if crate::paginacao_controle::paginador_com_prefixo(html, prefixo) {
            return Err(ErroRetornos::ListaDeRetornosComMaisDeUmaPagina);
        }
    }
    let dom = crate::tela_html::arvore(html);
    let mut situacoes = HashSet::new();
    let mut lista = Vec::new();
    for no in dom.elementos("table") {
        let Some(summary) = no.atributo("summary") else {
            continue;
        };
        let Some(situacao) = situacao_da_tabela(summary) else {
            continue;
        };
        if !situacoes.insert(situacao) || no.elementos("table").next().is_some() {
            return Err(ErroRetornos::TelaDeRetornosDesconhecida);
        }
        lista.extend(ler_tabela(no, situacao)?);
    }
    // A lista mensal completa observada em todas as sete instalacoes pode
    // omitir "Para Devolver", mas so dentro da area mensal reconhecida.
    // Tabela solta sem essa moldura nao comprova completude.
    let mut vazios = HashSet::new();
    let mut areas_aguardando = 0usize;
    let mut areas_observadas = HashSet::new();
    let mut area_aguardando_com_tabela = false;
    for no in dom.elementos("div") {
        let situacao = match no.atributo("id") {
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
            for no in no.elementos("table") {
                if let Some(summary) = no.atributo("summary")
                    && let Some(tipo) = situacao_da_tabela(summary)
                {
                    reconhecidas.push(tipo);
                }
            }
            area_aguardando_com_tabela = reconhecidas == ["aguardando"];
        }
        if identificador(&texto(no)) == "nenhum registro encontrado." && !vazios.insert(situacao) {
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
            "<table summary='{situacao}'><caption>1 registro:</caption><thead><tr><th>Processo</th><th>Unidade</th><th>Data Programada</th><th>Data de Retorno Efetiva</th><th>Prazo Restante</th></tr></thead><tbody><tr><td><a>{numero}</a></td><td>{unidade}</td><td>15/10/2026</td><td></td><td></td></tr></tbody></table>"
        )
    }
    #[test]
    fn mes_com_duas_situacoes_preserva_linhas_e_ordem() {
        // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}{}",
            tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1"),
            tabela("Para Devolver", "NUP-B", "U2")
        ));
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
        let html = crate::tela_html::HtmlServido::de_teste(
            "<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>",
        );
        assert_eq!(extrair_retornos(&html).unwrap(), Vec::<Retorno>::new());
        assert_eq!(
            extrair_retornos(&crate::tela_html::HtmlServido::de_teste(
                "<div>outra tela</div>"
            )),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn legenda_maior_que_as_linhas_nao_publica_resultado_parcial() {
        let html = crate::tela_html::HtmlServido::de_teste(
            &tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
                .replace("1 registro:", "2 registros:"),
        );
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::ListaDeRetornosIncompleta)
        );
    }
    #[test]
    fn tabelas_duplicadas_sao_recusadas() {
        let t = tabela("Para Devolver", "NUP-A", "U1");
        assert_eq!(
            extrair_retornos(&crate::tela_html::HtmlServido::de_teste(&format!("{t}{t}"))),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn pagina_apresenta_paginador_sem_replay_falha_fechado() {
        let html = crate::tela_html::HtmlServido::de_teste(
            &(tabela("Para Devolver", "NUP-A", "U1")
                + "<a id='lnkDevolverProximaPaginaSuperior'>proxima</a>"),
        );
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::ListaDeRetornosComMaisDeUmaPagina)
        );
    }
    #[test]
    fn tabela_unica_sem_evidencia_da_outra_situacao_e_incompleta() {
        // Na UnB, as duas situacoes tem secoes independentes; nao publicar metade do mes.
        let html = crate::tela_html::HtmlServido::de_teste(&tabela(
            "Aguardando Retorno de Outras Unidades",
            "NUP-A",
            "U1",
        ));
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::ListaDeRetornosIncompleta)
        );
    }
    #[test]
    fn area_mensal_observada_com_aguardando_nao_inventa_devolver() {
        // Replay UnB/GO/PEN3: o mes completo pode omitir a tabela Para Devolver.
        // Uma tabela solta sem a area mensal continua recusada.
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "<div id='divAguardandoAreaTabela'>{}<table><tr><td>layout</td></tr></table></div>",
            tabela(
                "Tabela de processos aguardando retorno de outras unidades",
                "NUP-A",
                "U1"
            )
        ));
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].situacao, "aguardando");
    }
    #[test]
    fn tabela_unica_com_outra_area_explicitamente_vazia_pode_ser_lida() {
        // A ausencia da outra tabela so vale se a sua area disser que nao ha registros.
        let html = crate::tela_html::HtmlServido::de_teste(
            &(tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
                + "<div id='divDevolverAreaTabela'>Nenhum registro encontrado.</div>"),
        );
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].situacao, "aguardando");
    }
    #[test]
    fn area_aguardando_duplicada_nao_parece_mes_vazio_completo() {
        // Estrutura sintética: area repetida torna ambigua a completude do mes.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>                    <div id='divAguardandoAreaTabela'>Area pendente de carregamento</div>",
        );
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn area_devolver_duplicada_nao_parece_mes_completo() {
        // O aviso de vazio de uma area nao prova a integridade da outra duplicata.
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}<div id='divDevolverAreaTabela'>Nenhum registro encontrado.</div>             <div id='divDevolverAreaTabela'>Area pendente de carregamento</div>",
            tabela("Aguardando Retorno de Outras Unidades", "NUP-A", "U1")
        ));
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
    }
    #[test]
    fn data_fora_do_calendario_nao_e_publicada() {
        let html = crate::tela_html::HtmlServido::de_teste(
            &tabela("Para Devolver", "NUP-A", "U1").replace("15/10/2026", "31/02/2026"),
        );
        assert_eq!(
            extrair_retornos(&html),
            Err(ErroRetornos::TelaDeRetornosDesconhecida)
        );
        assert!(data_valida("29/02/2024"));
        assert!(!data_valida("29/02/2025"));
    }
    fn duas_tabelas(celulas: &str) -> String {
        format!(
            "<div id='divAguardandoAreaTabela'><table summary='Aguardando Retorno de Outras Unidades'><caption>1 registro:</caption><thead><tr><th>Processo</th><th>Unidade</th><th>Data Programada</th><th>Data de Retorno Efetiva</th><th>Prazo Restante</th></tr></thead><tbody><tr><td><a>P1</a></td><td>U1</td><td>15/10/2026</td>{celulas}</tr></tbody></table></div>\
             <div id='divDevolverAreaTabela'><table summary='Para Devolver'><caption>0 registros:</caption><thead><tr><th>Processo</th><th>Unidade</th><th>Data Programada</th><th>Data de Retorno Efetiva</th><th>Prazo Restante</th></tr></thead></table></div>"
        )
    }
    #[test]
    fn celula_vazia_de_devolucao_e_prazo_nao_vira_null() {
        let html = crate::tela_html::HtmlServido::de_teste(&duas_tabelas("<td></td><td></td>"));
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(itens[0].retornado_em, DataDeRetorno::Pendente);
        assert_eq!(itens[0].prazo_restante, PrazoRestante::SemPrazo);
        let json = serde_json::to_value(&itens[0]).unwrap();
        assert_eq!(json["data_retorno"], "pendente");
        assert_eq!(json["prazo_restante"], "sem_prazo");
    }
    #[test]
    fn celulas_preenchidas_sao_lidas_como_fato() {
        let html = crate::tela_html::HtmlServido::de_teste(&duas_tabelas(
            "<td>16/10/2026</td><td>2 dias</td>",
        ));
        let itens = extrair_retornos(&html).unwrap();
        assert_eq!(
            itens[0].retornado_em,
            DataDeRetorno::Em("16/10/2026".to_owned())
        );
        assert_eq!(
            itens[0].prazo_restante,
            PrazoRestante::Em("2 dias".to_owned())
        );
    }
    #[test]
    fn coluna_ausente_e_recusada_com_erro_tipado() {
        let sem_devolucao =
            duas_tabelas("<td></td><td></td>").replacen("<th>Data de Retorno Efetiva</th>", "", 1);
        assert_eq!(
            extrair_retornos(&crate::tela_html::HtmlServido::de_teste(&sem_devolucao)),
            Err(ErroRetornos::ColunaDataDeRetornoAusente)
        );
        let sem_prazo =
            duas_tabelas("<td></td><td></td>").replacen("<th>Prazo Restante</th>", "", 1);
        assert_eq!(
            extrair_retornos(&crate::tela_html::HtmlServido::de_teste(&sem_prazo)),
            Err(ErroRetornos::ColunaPrazoRestanteAusente)
        );
    }
}
