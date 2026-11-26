//! Leitor da tela mista do catálogo da unidade (#880).
//! WIP: forma do oráculo Todos caabe699; ainda sem ligação à CLI.
use crate::legenda::{ErroLegenda, Faixa, ler_legenda};
use crate::tela_html::{HtmlServido, arvore};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Estado {
    Ativo,
    Inativo,
}

#[derive(Debug, PartialEq, Eq)]
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
    let tabela = tabelas
        .next()
        .ok_or(ErroCatalogo::TabelaAusenteOuAmbigua)?;
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
    let cabecalho = linhas
        .next()
        .ok_or(ErroCatalogo::CabecalhoNaoReconhecido)?;
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
        let mut estados = celula(acoes)?
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
    let faixa_observada = match legenda.faixa {
        Some(f)
            if f.inicio > 0
                && f.fim >= f.inicio
                && f.fim <= legenda.total
                && f.fim - f.inicio + 1 == itens.len() =>
        {
            Some(f)
        }
        None if legenda.total == itens.len() => (legenda.total > 0).then_some(Faixa {
            inicio: 1,
            fim: legenda.total,
        }),
        _ => return Err(ErroCatalogo::FaixaIncoerente),
    };
    Ok(Lista {
        total_declarado: legenda.total,
        faixa_observada,
        itens,
    })
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

    // Fonte original, separada do exemplo sintético e da inspeção de hashes.
    // O oráculo alternativo é o value do checkbox da linha, não o texto ID.
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
            let mut paginas = 0;
            for r in gravacao::respostas(&captura.json) {
                if r.operacao != "marcador_listar" || r.metodo != "GET" || r.status != 200 {
                    continue;
                }
                let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                let lista = ler(&html).unwrap();
                let dom = arvore(&html);
                let tabela = dom
                    .elementos("table")
                    .find(|e| e.atributo("summary") == Some("Tabela de Marcadores."))
                    .unwrap();
                let ids: Vec<_> = tabela
                    .elementos("input")
                    .filter(|e| e.atributo("type") == Some("checkbox"))
                    .map(|e| e.atributo("value").unwrap())
                    .collect();
                let obtidos: Vec<_> = lista.itens.iter().map(|m| m.id.as_str()).collect();
                assert_eq!(ids.len(), obtidos.len());
                assert!(ids.iter().zip(&obtidos).all(|(a, b)| a == b));
                let ativos = tabela
                    .elementos("img")
                    .filter(|e| e.atributo("title") == Some("Desativar Marcador"))
                    .count();
                let inativos = tabela
                    .elementos("img")
                    .filter(|e| e.atributo("title") == Some("Reativar Marcador"))
                    .count();
                assert_eq!(
                    ativos,
                    lista.itens.iter().filter(|m| m.estado == Estado::Ativo).count()
                );
                assert_eq!(
                    inativos,
                    lista.itens.iter().filter(|m| m.estado == Estado::Inativo).count()
                );
                paginas += 1;
            }
            assert!(paginas > 0, "nenhuma página de catálogo original foi lida");
        }
    }
}
