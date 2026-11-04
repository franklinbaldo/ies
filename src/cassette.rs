//! Replay offline seguro de resposta HTTP a partir de cassette pública.
use std::path::Path;
#[cfg(test)]
pub(crate) mod acervo;
mod conferencia;
pub(crate) use conferencia::ZipDoAcervo;
#[cfg(test)]
pub(crate) use conferencia::{conferir_cassette_ate, html_da_resposta};

fn campo<'a>(source: &'a str, field: &str) -> Option<&'a str> {
    let rest = source.strip_prefix("---\n")?;
    rest.split("\n---\n").next()?.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key == field).then_some(value.trim())
    })
}

fn legivel(bytes: &[u8], encoding: &str) -> Result<String, String> {
    let enc = encoding.to_ascii_lowercase();
    if enc.contains("utf-8") || enc == "utf8" {
        return String::from_utf8(bytes.to_vec())
            .map_err(|_| "resposta não é UTF-8 válido".to_owned());
    }
    if enc.contains("8859-1") || enc.contains("latin1") || enc.is_empty() {
        return Ok(bytes.iter().copied().map(char::from).collect());
    }
    Err("codificação de resposta ainda não suportada".into())
}

// O acervo OKF exige sufixo `.zip` exatamente em minúsculas.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn archive_filename(card: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(card).map_err(|_| "card ausente")?;
    let url = campo(&text, "source_zip").ok_or("origem ausente")?;
    let base = url
        .strip_prefix("https://archive.org/download/todos-sei-cassettes-2026/")
        .ok_or("origem fora do acervo")?;
    if base.contains("..") || !base.ends_with(".zip") {
        return Err("origem insegura".into());
    }
    let name = base.rsplit('/').next().ok_or("nome ausente")?;
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
    {
        return Err("nome de ZIP inválido".into());
    }
    Ok(name.to_owned())
}

/// O replay escolhe pelo conteúdo verificado da cassette, nunca pelo nome do cenário.
pub enum ReplayLido {
    Caixa(Vec<String>, String),
    Historico(String, String),
}

pub fn read_cassette_auto(
    card: &Path,
    archive: &Path,
    passos: &[usize],
    todas_paginas: bool,
) -> Result<ReplayLido, String> {
    let (pages, origem, operacao) =
        read_verified_html(card, archive, passos, None, todas_paginas, false)?;
    if operacao == "procedimento_consultar_historico" {
        let mut pages = pages;
        Ok(ReplayLido::Historico(pages.remove(0), origem))
    } else {
        Ok(ReplayLido::Caixa(pages, origem))
    }
}

pub fn read_cassette_document(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<(String, String), String> {
    let (mut pages, instalacao, _) = read_verified_html(
        card,
        archive,
        passos,
        Some("documento_imprimir_web"),
        false,
        false,
    )?;
    if pages.len() != 1 {
        return Err("um único HTML do documento deve ser selecionado".into());
    }
    Ok((pages.remove(0), instalacao))
}

/// Um único GET de histórico de versões já capturado e conferido por hash.
/// A consulta não executa links do histórico nem abre editor no SEI.
pub fn read_cassette_document_versions(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<(String, String), String> {
    let (mut pages, instalacao, _) = read_verified_html(
        card,
        archive,
        passos,
        Some("documento_versao_listar"),
        false,
        false,
    )?;
    if pages.len() != 1 {
        return Err("histórico de versões exige uma única resposta GET".into());
    }
    Ok((pages.remove(0), instalacao))
}

/// Seleção de todas as respostas GET de documento, preservando revisão e host
/// verificados. A identidade por número é decidida depois, pelo título SEI.
pub fn read_cassette_documents(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<(Vec<String>, String), String> {
    let (pages, instalacao, _) = read_verified_html(
        card,
        archive,
        passos,
        Some("documento_imprimir_web"),
        false,
        true,
    )?;
    Ok((pages, instalacao))
}

/// Lê somente uma resposta POST de pesquisa observada e verificada offline.
pub fn read_cassette_search_result(
    card: &Path,
    archive: &Path,
    passo: usize,
) -> Result<(String, String), String> {
    let (mut paginas, instalacao, _) = read_verified_html(
        card,
        archive,
        &[passo],
        Some("protocolo_pesquisar"),
        false,
        false,
    )?;
    Ok((paginas.remove(0), instalacao))
}

/// Lê múltiplas respostas POST de pesquisa com a mesma validação integral da
/// cassette e da instalação. Não infere sequencialidade nem cobertura de páginas.
pub fn read_cassette_search_results(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<(Vec<String>, String), String> {
    let (pages, instalacao, _) = read_verified_html(
        card,
        archive,
        passos,
        Some("protocolo_pesquisar"),
        false,
        true,
    )?;
    Ok((pages, instalacao))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroSelecaoOperacao {
    InteracoesAusentes,
    PassoNaoEncontrado { indice: usize },
    OperacaoDoPassoNaoSuportada { indice: usize },
    PassosComOperacoesDiferentes,
    HistoricoComVariasRespostas,
}
impl ErroSelecaoOperacao {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::InteracoesAusentes => "interacoes_da_cassette_ausentes",
            Self::PassoNaoEncontrado { .. } => "passo_da_cassette_ausente",
            Self::OperacaoDoPassoNaoSuportada { .. } => "operacao_do_passo_nao_suportada",
            Self::PassosComOperacoesDiferentes => "passos_de_operacoes_diferentes",
            Self::HistoricoComVariasRespostas => "historico_exige_um_passo",
        }
    }
}

fn operation_for_replay(
    json: &serde_json::Value,
    passos: &[usize],
) -> Result<&'static str, ErroSelecaoOperacao> {
    let interactions = json["interactions"]
        .as_array()
        .ok_or(ErroSelecaoOperacao::InteracoesAusentes)?;

    // Seleção explícita tem precedência sobre outras operações da cassette.
    // Não deduzir o tipo pelo nome do cenário nem misturar respostas.
    if !passos.is_empty() {
        let mut escolhido = None;
        for &indice in passos {
            let passo = interactions
                .get(indice)
                .ok_or(ErroSelecaoOperacao::PassoNaoEncontrado { indice })?;
            let operacao = match passo["request"]["operation"].as_str() {
                Some("procedimento_controlar") => "procedimento_controlar",
                Some("procedimento_consultar_historico") => "procedimento_consultar_historico",
                _ => return Err(ErroSelecaoOperacao::OperacaoDoPassoNaoSuportada { indice }),
            };
            if escolhido.is_some_and(|anterior| anterior != operacao) {
                return Err(ErroSelecaoOperacao::PassosComOperacoesDiferentes);
            }
            escolhido = Some(operacao);
        }
        if escolhido == Some("procedimento_consultar_historico") && passos.len() != 1 {
            return Err(ErroSelecaoOperacao::HistoricoComVariasRespostas);
        }
        return Ok(escolhido.expect("passos não vazios"));
    }

    // No modo automático, histórico é preferido somente quando observado
    // em GET; a cardinalidade é conferida pelo leitor de páginas.
    let has_history = interactions.iter().any(|it| {
        it["request"]["operation"] == "procedimento_consultar_historico"
            && it["request"]["method"] == "GET"
    });
    Ok(if has_history {
        "procedimento_consultar_historico"
    } else {
        "procedimento_controlar"
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroPaginasAutomaticas {
    SemInteracoes,
    SemPaginasDoControle,
    PrimeiraPaginaNaoEGet,
    PaginasDeBlocosDiferentes,
}

impl ErroPaginasAutomaticas {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::SemInteracoes => "interacoes_da_cassette_ausentes",
            Self::SemPaginasDoControle => "paginas_do_controle_ausentes",
            Self::PrimeiraPaginaNaoEGet => "primeira_pagina_do_controle_nao_e_get",
            Self::PaginasDeBlocosDiferentes => "paginas_do_controle_intercaladas",
        }
    }
}

fn passos_de_todas_as_paginas(
    json: &serde_json::Value,
) -> Result<Vec<usize>, ErroPaginasAutomaticas> {
    let interacoes = json["interactions"]
        .as_array()
        .ok_or(ErroPaginasAutomaticas::SemInteracoes)?;
    let indices: Vec<usize> = interacoes
        .iter()
        .enumerate()
        .filter_map(|(indice, passo)| {
            (passo["request"]["operation"] == "procedimento_controlar").then_some(indice)
        })
        .collect();
    let Some(&inicio) = indices.first() else {
        return Err(ErroPaginasAutomaticas::SemPaginasDoControle);
    };
    if interacoes[inicio]["request"]["method"] != "GET" {
        return Err(ErroPaginasAutomaticas::PrimeiraPaginaNaoEGet);
    }
    if indices.windows(2).any(|par| par[1] != par[0] + 1) {
        return Err(ErroPaginasAutomaticas::PaginasDeBlocosDiferentes);
    }
    Ok(indices)
}

fn read_verified_html(
    card: &Path,
    archive: &Path,
    passos: &[usize],
    operation: Option<&str>,
    todas_paginas: bool,
    permitir_multiplos: bool,
) -> Result<(Vec<String>, String, String), String> {
    let conferida = conferencia::conferir_cassette(card, &ZipDoAcervo::ler(archive)?)?;
    let (json, installation) = (conferida.json, conferida.instalacao);
    let operation = if todas_paginas {
        "procedimento_controlar"
    } else {
        match operation {
            Some(operation) => operation,
            None => {
                operation_for_replay(&json, passos).map_err(|erro| erro.subcodigo().to_owned())?
            }
        }
    };
    let pages = selected_html_pages(&json, passos, operation, todas_paginas, permitir_multiplos)?;
    Ok((pages, installation, operation.to_owned()))
}

fn selected_html_pages(
    json: &serde_json::Value,
    passos: &[usize],
    operation: &str,
    todas_paginas: bool,
    permitir_multiplos: bool,
) -> Result<Vec<String>, String> {
    let list = json["interactions"]
        .as_array()
        .ok_or("interações ausentes")?;
    let chosen: Vec<usize> = if todas_paginas {
        passos_de_todas_as_paginas(json).map_err(|erro| erro.subcodigo().to_owned())?
    } else if passos.is_empty() {
        list.iter()
            .enumerate()
            .filter_map(|(i, it)| {
                (it["request"]["operation"] == operation && it["request"]["method"] == "GET")
                    .then_some(i)
            })
            .collect()
    } else {
        passos.to_vec()
    };
    if chosen.is_empty() || chosen.len() > 200 {
        return Err("quantidade de páginas inválida".into());
    }
    if passos.is_empty() && chosen.len() != 1 && !permitir_multiplos && !todas_paginas {
        return Err("cassette com várias caixas: selecione os passos explicitamente".into());
    }
    let mut seen = std::collections::HashSet::new();
    if !chosen.iter().all(|i| seen.insert(*i)) {
        return Err("passo repetido na seleção".into());
    }
    let mut pages = Vec::with_capacity(chosen.len());
    let mut total_bytes = 0usize;
    for (order, step) in chosen.into_iter().enumerate() {
        let it = list.get(step).ok_or("passo inexistente")?;
        let method = it["request"]["method"].as_str();
        let metodo_esperado = if operation == "protocolo_pesquisar" {
            method == Some("POST")
        } else {
            method == Some("GET")
                || (operation == "procedimento_controlar" && order > 0 && method == Some("POST"))
        };
        if it["request"]["operation"] != operation
            || !metodo_esperado
            || it["response"]["status_code"] != 200
        {
            return Err("passo não é uma resposta de caixa admitida (GET inicial; GET/POST subsequentes offline)".into());
        }
        let (bytes, encoding) = conferencia::corpo_html(it)?;
        total_bytes += bytes.len();
        if total_bytes > 48_000_000 {
            return Err("conjunto de HTML maior que 48 MB".into());
        }
        pages.push(legivel(&bytes, &encoding)?);
    }
    Ok(pages)
}

pub fn decode_http_body(bytes: &[u8], content_type: &str) -> Result<String, String> {
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err("HTML maior que 8 MB".into());
    }
    let enc = if content_type.to_ascii_lowercase().contains("8859-1") {
        "iso-8859-1"
    } else {
        "utf-8"
    };
    legivel(bytes, enc)
}

#[cfg(test)]
#[path = "cassette/tests.rs"]
mod replay_operation_tests;
