//! Leitor da tela mista do catálogo da unidade (#880).
//! Forma mista observada no oráculo Todos caabe699 e nas cassettes UnB/PEN1.
use crate::legenda::{ErroLegenda, Faixa, ler_legenda};
use crate::tela_html::{HtmlServido, arvore};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Estado {
    Ativo,
    Inativo,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Marcador {
    pub(crate) id: String,
    pub(crate) nome: String,
    pub(crate) cor: String,
    pub(crate) estado: Estado,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Lista {
    pub(crate) total_declarado: usize,
    pub(crate) faixa_observada: Option<Faixa>,
    pub(crate) itens: Vec<Marcador>,
}

/// Projeção da página, não uma contagem do catálogo inteiro por estado.
#[derive(Debug, Serialize)]
pub(crate) struct Selecao {
    pub(crate) total_declarado: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) faixa_observada: Option<Faixa>,
    pub(crate) total_observado: usize,
    pub(crate) estado: Estado,
    pub(crate) itens: Vec<Marcador>,
}

impl Lista {
    pub(crate) fn selecionar(self, estado: Estado) -> Selecao {
        let total_observado = self.itens.len();
        Selecao {
            total_declarado: self.total_declarado,
            faixa_observada: self.faixa_observada,
            total_observado,
            estado,
            itens: self
                .itens
                .into_iter()
                .filter(|m| m.estado == estado)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCatalogo {
    FiltroPersistente,
    TabelaAusenteOuAmbigua,
    CabecalhoNaoReconhecido,
    LegendaAusenteOuAmbigua,
    Legenda(#[from] ErroLegenda),
    FaixaIncoerente,
    LinhaForaDaForma,
    IdRepetido,
    EstadoNaoDemonstrado,
}

/// Só a forma mista. O total/faixa referem-se à tela, não ao subconjunto
/// de ativos ou inativos que a CLI futuramente selecionará.
pub(crate) fn ler(html: &HtmlServido) -> Result<Lista, ErroCatalogo> {
    let dom = arvore(html);
    if dom.elementos("input").any(|e| {
        e.atributo("id") == Some("hdnMarcadoresFiltro")
            || e.atributo("name") == Some("hdnMarcadoresFiltro")
    }) {
        return Err(ErroCatalogo::FiltroPersistente);
    }
    let mut tabelas = dom
        .elementos("table")
        .filter(|e| e.atributo("summary") == Some("Tabela de Marcadores."));
    let tabela = tabelas.next().ok_or(ErroCatalogo::TabelaAusenteOuAmbigua)?;
    if tabelas.next().is_some() {
        return Err(ErroCatalogo::TabelaAusenteOuAmbigua);
    }
    let mut captions = tabela.elementos("caption");
    let legenda = captions
        .next()
        .ok_or(ErroCatalogo::LegendaAusenteOuAmbigua)?;
    if captions.next().is_some() {
        return Err(ErroCatalogo::LegendaAusenteOuAmbigua);
    }
    let legenda = ler_legenda(&legenda.texto())?;
    let mut linhas = tabela.elementos("tr");
    let cabecalho = linhas.next().ok_or(ErroCatalogo::CabecalhoNaoReconhecido)?;
    let nomes: Vec<_> = cabecalho
        .filhos()
        .filter(|e| e.nome() == "th")
        .map(|e| e.texto().trim().to_owned())
        .collect();
    let coluna = |nome: &str| {
        let mut posicoes = nomes.iter().enumerate().filter(|(_, n)| *n == nome);
        let (i, _) = posicoes
            .next()
            .ok_or(ErroCatalogo::CabecalhoNaoReconhecido)?;
        if posicoes.next().is_some() {
            return Err(ErroCatalogo::CabecalhoNaoReconhecido);
        }
        Ok(i)
    };
    let (id, nome, cor, acoes) = (
        coluna("ID")?,
        coluna("Nome")?,
        coluna("Ícone")?,
        coluna("Ações")?,
    );
    let mut itens = Vec::new();
    let mut ids = HashSet::new();
    for linha in linhas {
        let celulas: Vec<_> = linha.filhos().filter(|e| e.nome() == "td").collect();
        if celulas.len() != nomes.len() {
            return Err(ErroCatalogo::LinhaForaDaForma);
        }
        let celula = |i| {
            celulas
                .get(i)
                .copied()
                .ok_or(ErroCatalogo::LinhaForaDaForma)
        };
        let id = celula(id)?.texto().trim().to_owned();
        let nome = celula(nome)?.texto().trim().to_owned();
        let cor = celula(cor)?
            .elementos("img")
            .next()
            .and_then(|e| e.atributo("title"))
            .filter(|s| !s.is_empty())
            .ok_or(ErroCatalogo::LinhaForaDaForma)?
            .to_owned();
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || nome.is_empty() {
            return Err(ErroCatalogo::LinhaForaDaForma);
        }
        if !ids.insert(id.clone()) {
            return Err(ErroCatalogo::IdRepetido);
        }
        let mut estados =
            celula(acoes)?
                .elementos("img")
                .filter_map(|e| match e.atributo("title") {
                    Some("Desativar Marcador") => Some(Estado::Ativo),
                    Some("Reativar Marcador") => Some(Estado::Inativo),
                    _ => None,
                });
        let estado = estados.next().ok_or(ErroCatalogo::EstadoNaoDemonstrado)?;
        if estados.next().is_some() {
            return Err(ErroCatalogo::EstadoNaoDemonstrado);
        }
        itens.push(Marcador {
            id,
            nome,
            cor,
            estado,
        });
    }
    let faixa_observada = conferir_faixa(legenda.total, legenda.faixa, itens.len())?;
    Ok(Lista {
        total_declarado: legenda.total,
        faixa_observada,
        itens,
    })
}

fn conferir_faixa(
    total: usize,
    faixa: Option<Faixa>,
    observados: usize,
) -> Result<Option<Faixa>, ErroCatalogo> {
    match faixa {
        Some(f)
            if f.inicio > 0
                && f.fim >= f.inicio
                && f.fim <= total
                && f.fim - f.inicio + 1 == observados =>
        {
            Ok(Some(f))
        }
        None if total == observados => Ok((total > 0).then_some(Faixa {
            inicio: 1,
            fim: total,
        })),
        _ => Err(ErroCatalogo::FaixaIncoerente),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MISTA: &str = r#"<table summary="Tabela de Marcadores.">
    <caption>Lista de Marcadores (2 registros):</caption>
    <tr><th></th><th>Nome</th><th>Ações</th><th>ID</th><th>Ícone</th></tr>
    <tr><td></td><td>Mesmo #nome</td><td><img title="Desativar Marcador"></td><td>01</td><td><img title="Branco"></td></tr>
    <tr><td></td><td>Mesmo #nome</td><td><img title="Reativar Marcador"></td><td>02</td><td><img title="Amarelo"></td></tr>
    </table>"#;

    fn ler_texto(texto: &str) -> Result<Lista, ErroCatalogo> {
        ler(&HtmlServido::de_teste(texto))
    }

    #[test]
    fn homonimos_e_estados_preservam_identidades() {
        let lista = ler_texto(MISTA).unwrap();
        assert_eq!(lista.total_declarado, 2);
        assert_eq!(
            lista
                .itens
                .iter()
                .map(|m| (m.id.as_str(), m.nome.as_str(), m.cor.as_str(), m.estado))
                .collect::<Vec<_>>(),
            [
                ("01", "Mesmo #nome", "Branco", Estado::Ativo),
                ("02", "Mesmo #nome", "Amarelo", Estado::Inativo)
            ]
        );
    }

    #[test]
    fn ausencia_ou_contradicao_nao_significa_ativo() {
        for acao in ["", "Alterar Marcador"] {
            assert_eq!(
                ler_texto(&MISTA.replace("Reativar Marcador", acao)),
                Err(ErroCatalogo::EstadoNaoDemonstrado)
            );
        }
        let contradicao = MISTA.replace(
            "<img title=\"Reativar Marcador\">",
            "<img title=\"Reativar Marcador\"><img title=\"Desativar Marcador\">",
        );
        assert_eq!(
            ler_texto(&contradicao),
            Err(ErroCatalogo::EstadoNaoDemonstrado)
        );
    }

    #[test]
    fn tabela_ausente_filtro_e_total_divergente_nao_sao_vazio() {
        assert_eq!(
            ler_texto("<div/>"),
            Err(ErroCatalogo::TabelaAusenteOuAmbigua)
        );
        let filtrada = format!("<input name='hdnMarcadoresFiltro' value='S'>{MISTA}");
        assert_eq!(ler_texto(&filtrada), Err(ErroCatalogo::FiltroPersistente));
        assert_eq!(
            ler_texto(&MISTA.replace("2 registros", "3 registros")),
            Err(ErroCatalogo::FaixaIncoerente)
        );
        assert_eq!(
            ler_texto(&MISTA.replace("<td>02</td>", "<td>01</td>")),
            Err(ErroCatalogo::IdRepetido)
        );
    }

    #[test]
    fn pagina_parcial_preserva_total_e_faixa() {
        let parcial = MISTA.replace("2 registros", "4 registros - 2 a 3");
        let lista = ler_texto(&parcial).unwrap();
        assert_eq!(lista.total_declarado, 4);
        assert_eq!(lista.faixa_observada, Some(Faixa { inicio: 2, fim: 3 }));
        assert_eq!(lista.itens.len(), 2);
    }

    #[test]
    fn vazio_explicito_preserva_total_zero() {
        let vazio = r#"<table summary="Tabela de Marcadores.">
        <caption>Lista de Marcadores (0 registros):</caption>
        <tr><th></th><th>Nome</th><th>Ações</th><th>ID</th><th>Ícone</th></tr>
        </table>"#;
        let lista = ler_texto(vazio).unwrap();
        assert_eq!(lista.total_declarado, 0);
        assert_eq!(lista.faixa_observada, None);
        assert_eq!(lista.itens, [] as [Marcador; 0]);
        assert_eq!(
            serde_json::to_value(lista.selecionar(Estado::Ativo)).unwrap(),
            serde_json::json!({
                "total_declarado": 0,
                "total_observado": 0,
                "estado": "ativo",
                "itens": []
            })
        );
    }

    #[test]
    fn selecao_nao_confunde_catalogo_pagina_e_subconjunto() {
        let parcial = MISTA.replace("2 registros", "4 registros - 2 a 3");
        for (estado, id, rotulo) in [
            (Estado::Ativo, "01", "ativo"),
            (Estado::Inativo, "02", "inativo"),
        ] {
            let selecao = ler_texto(&parcial).unwrap().selecionar(estado);
            let json = serde_json::to_value(selecao).unwrap();
            assert_eq!(
                json,
                serde_json::json!({
                    "total_declarado": 4,
                    "faixa_observada": {"inicio": 2, "fim": 3},
                    "total_observado": 2,
                    "estado": rotulo,
                    "itens": [{
                        "id": id,
                        "nome": "Mesmo #nome",
                        "cor": if estado == Estado::Ativo { "Branco" } else { "Amarelo" },
                        "estado": rotulo
                    }]
                })
            );
        }
    }

    #[test]
    fn subconjunto_vazio_na_pagina_nao_e_catalogo_vazio() {
        let parcial = MISTA
            .replace("2 registros", "4 registros - 2 a 3")
            .replace("Reativar Marcador", "Desativar Marcador");
        let selecao = ler_texto(&parcial).unwrap().selecionar(Estado::Inativo);
        assert_eq!(selecao.total_declarado, 4);
        assert_eq!(selecao.total_observado, 2);
        assert_eq!(selecao.faixa_observada, Some(Faixa { inicio: 2, fim: 3 }));
        assert_eq!(selecao.itens, [] as [Marcador; 0]);
    }

    // Leitura de testemunhas diretamente da linha original, sem chamar ler()
    // ou ler_legenda(). O checkbox fornece uma identidade independente.
    fn testemunhas_catalogo(html: &HtmlServido, lista: &Lista) {
        let dom = arvore(html);
        let tabela = dom
            .elementos("table")
            .find(|e| e.atributo("summary") == Some("Tabela de Marcadores."))
            .unwrap();
        let legenda = tabela.elementos("caption").next().unwrap().texto();
        let numeros: Vec<usize> = legenda
            .split(|c: char| !c.is_ascii_digit() && c != '.')
            .filter(|s| s.bytes().any(|b| b.is_ascii_digit()))
            .map(|s| s.replace('.', "").parse().unwrap())
            .collect();
        let (total, faixa) = match numeros.as_slice() {
            [0] => (0, None),
            [total] => (
                *total,
                Some(Faixa {
                    inicio: 1,
                    fim: *total,
                }),
            ),
            [total, inicio, fim] => (
                *total,
                Some(Faixa {
                    inicio: *inicio,
                    fim: *fim,
                }),
            ),
            _ => panic!("legenda original fora da forma testemunhada"),
        };
        assert_eq!(lista.total_declarado, total);
        assert_eq!(lista.faixa_observada, faixa);

        let mut linhas = tabela.elementos("tr");
        let cabecalhos: Vec<_> = linhas
            .next()
            .unwrap()
            .filhos()
            .filter(|e| e.nome() == "th")
            .map(|e| e.texto())
            .collect();
        let esperados: Vec<_> = linhas
            .filter_map(|linha| {
                let checkbox = linha.elementos("input").find(|e| {
                    e.atributo("type") == Some("checkbox") && e.atributo("value").is_some()
                })?;
                let id = checkbox.atributo("value").unwrap();
                let celulas: Vec<_> = linha.filhos().filter(|e| e.nome() == "td").collect();
                let mut nome = None;
                let mut cor = None;
                for (titulo, celula) in cabecalhos.iter().zip(celulas) {
                    match titulo.trim() {
                        "Nome" => nome = Some(celula.texto().trim().to_owned()),
                        "Ícone" => {
                            cor = celula.elementos("img").find_map(|e| e.atributo("title"));
                        }
                        _ => {}
                    }
                }
                let ativo = linha
                    .elementos("img")
                    .any(|e| e.atributo("title") == Some("Desativar Marcador"));
                let inativo = linha
                    .elementos("img")
                    .any(|e| e.atributo("title") == Some("Reativar Marcador"));
                assert_ne!(ativo, inativo, "estado original ambíguo");
                let estado = if ativo {
                    Estado::Ativo
                } else {
                    Estado::Inativo
                };
                Some((id, nome.unwrap(), cor.unwrap(), estado))
            })
            .collect();
        let obtidos: Vec<_> = lista
            .itens
            .iter()
            .map(|m| (m.id.as_str(), m.nome.clone(), m.cor.as_str(), m.estado))
            .collect();
        assert_eq!(esperados, obtidos);
    }

    // Fonte original, separada do exemplo sintético e da inspeção de hashes.
    // O oráculo confronta identidade, nome, cor, estado, total e faixa.
    #[test]
    #[ignore = "ZIPs originais no IES_ACERVO_CACHE; offline"]
    fn cassette_catalogo_misto_unb_pen1() {
        use crate::cassette::acervo::gravacao;
        use crate::cassette::{
            ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta,
        };
        let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
            || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
            std::path::PathBuf::from,
        );
        for caminho in [
            "cassettes/unb/marcador-listar-estados-unb--c47ad7e28f89.md",
            "cassettes/pen1/marcador-listar-estados-pen1--8358611b6263.md",
        ] {
            let card = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(caminho);
            let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
            let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
            use iesei::gestao_marcadores as gestao;
            let origem = gravacao::instalacao(&captura.json).unwrap();
            let mut banco = rusqlite::Connection::open_in_memory().unwrap();
            banco.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
            banco
                .execute_batch(include_str!(
                    "../crates/iesei/migrations/0043_gestao_marcadores.sql"
                ))
                .unwrap();
            // Chave de ensaio, não atribuição de unidade real: a gravação pode
            // omitir a barra da unidade ou conter troca de unidade.
            let unidade = "UNIDADE-DE-ENSAIO";
            let mut paginas = Vec::new();
            for r in gravacao::respostas(&captura.json) {
                if r.operacao != "marcador_listar" || r.metodo != "GET" || r.status != 200 {
                    continue;
                }
                let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                let lista = ler(&html).unwrap();
                testemunhas_catalogo(&html, &lista);
                let pagina = gestao::Pagina {
                    total: lista.total_declarado,
                    faixa: lista.faixa_observada.map(|f| (f.inicio, f.fim)),
                    itens: lista
                        .itens
                        .into_iter()
                        .map(|m| gestao::Marcador {
                            id: m.id,
                            nome: m.nome,
                            cor: m.cor,
                            estado: match m.estado {
                                Estado::Ativo => gestao::Estado::Ativo,
                                Estado::Inativo => gestao::Estado::Inativo,
                            },
                        })
                        .collect(),
                };
                gestao::registrar(&mut banco, &origem, caminho, unidade, r.passo, &pagina).unwrap();
                paginas.push((r.passo, pagina));
            }
            assert!(
                !paginas.is_empty(),
                "nenhuma página de catálogo original foi lida"
            );
            assert_eq!(
                gestao::ler_selecionada(&banco, &origem, caminho, unidade).unwrap(),
                None
            );
            // Selecionar em ordem inversa verifica que uma página posterior
            // não substituiu a anterior, inclusive se seus IDs coincidirem.
            for (ordinal, pagina) in paginas.into_iter().rev() {
                assert!(gestao::selecionar(&banco, &origem, caminho, unidade, ordinal).unwrap());
                assert_eq!(
                    gestao::ler_selecionada(&banco, &origem, caminho, unidade).unwrap(),
                    Some(pagina)
                );
            }
        }
    }
}
