//! Despacho do replay offline de cassettes OKF, sem acesso ao SEI.
use crate::replay_args::Replay;
use crate::{
    cassette, erro_fonte, erro_lista, historico_cli, replay_busca_documento, replay_consulta,
    replay_documento_cli, replay_pesquisa_cli, replay_versoes_cli, unidade_lista_cli,
};
use std::path::PathBuf;
use std::process::ExitCode;

fn nome_cenario_aceito(cenario: &str) -> bool {
    !cenario.is_empty()
        && !cenario.contains("..")
        && cenario
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn localizar_cards(pasta: &std::path::Path, cenario: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut encontrados = Vec::new();
    for dir in std::fs::read_dir(pasta)?.flatten() {
        if !dir.path().is_dir() {
            continue;
        }
        if let Ok(cards) = std::fs::read_dir(dir.path()) {
            for file in cards.flatten() {
                let name = file.file_name();
                let name = name.to_string_lossy();
                if name == format!("{cenario}.md")
                    || name.starts_with(&format!("{cenario}--")) && name.ends_with(".md")
                {
                    encontrados.push(file.path());
                }
            }
        }
    }
    Ok(encontrados)
}

fn executar_replay_documento(
    card: &std::path::Path,
    zip: &std::path::Path,
    replay: &Replay,
    json_mode: bool,
) -> ExitCode {
    if replay.historico_versoes {
        return replay_versoes_cli::mostrar(
            card,
            zip,
            &replay.passo,
            replay.numero_documento.as_deref(),
            replay.id_documento_interno.as_deref(),
            replay.versao_documento,
            json_mode,
        );
    }
    if let Some(termo) = replay.termo_documento.as_deref() {
        return replay_busca_documento::mostrar(card, zip, &replay.passo, termo, json_mode);
    }
    replay_documento_cli::mostrar(
        card,
        zip,
        &replay.passo,
        replay.numero_documento.as_deref(),
        replay.listar_documentos,
        &replay_documento_cli::SaidaDocumento {
            texto: replay.texto_documento,
            arquivo: replay.arquivo_texto.as_deref(),
        },
        json_mode,
    )
}

pub(crate) fn executar_replay(replay: &Replay, unidade: Option<&str>, json_mode: bool) -> ExitCode {
    if !nome_cenario_aceito(&replay.cenario) {
        return erro_lista(json_mode, "entrada_invalida", "nome de cenário inválido", 2);
    }
    let cenario = &replay.cenario;
    let pasta = replay.acervo.join("cassettes");
    let Ok(encontrados) = localizar_cards(&pasta, cenario) else {
        return erro_fonte(
            json_mode,
            "acervo_ausente",
            "diretório de cassettes não encontrado",
        );
    };
    if encontrados.len() != 1 {
        return erro_fonte(
            json_mode,
            "cenario_ambiguo_ou_ausente",
            "cenário não encontrado ou com várias revisões; use um identificador mais específico",
        );
    }
    let card = &encontrados[0];
    let Ok(zip_name) = cassette::archive_filename(card) else {
        return erro_fonte(json_mode, "card_invalido", "metadados de origem inválidos");
    };
    let zip = replay.acervo.join(".cache/ies").join(zip_name);
    if !zip.is_file() {
        return erro_fonte(
            json_mode,
            "zip_ausente",
            "ZIP não localizado no cache .cache/ies; obtenha o arquivo indicado no card OKF",
        );
    }
    if replay.resultado_pesquisa {
        return replay_pesquisa_cli::mostrar(
            card,
            &zip,
            &replay.passo,
            replay.numero_processo.as_deref(),
            replay.unir_resultados_pesquisa,
            json_mode,
        );
    }
    if replay.documento && (replay.numero_processo.is_some() || replay.todas_paginas) {
        return erro_lista(
            json_mode,
            "opcoes_incompativeis",
            "documento não é caixa de processos",
            2,
        );
    }
    if replay.documento {
        return executar_replay_documento(card, &zip, replay, json_mode);
    }
    match cassette::read_cassette_auto(card, &zip, &replay.passo, replay.todas_paginas) {
        Ok(cassette::ReplayLido::Historico(html, origem)) => historico_cli::mostrar_historico(
            &html,
            &origem,
            replay.numero_processo.as_deref(),
            replay.termo_andamento.as_deref(),
            json_mode,
        ),
        Ok(cassette::ReplayLido::Caixa(htmls, origem)) => {
            if replay.termo_andamento.is_some() {
                return historico_cli::erro_filtro_em_caixa(json_mode);
            }
            if let Some(numero) = replay.numero_processo.as_deref() {
                replay_consulta::mostrar(&htmls, &origem, numero, unidade, json_mode)
            } else {
                unidade_lista_cli::mostrar_caixa(
                    &htmls,
                    &origem,
                    unidade,
                    json_mode,
                    replay.todas_paginas,
                    None,
                )
            }
        }
        Err(_) => erro_fonte(
            json_mode,
            "cassette_invalida",
            "replay não verificável para as operações registradas",
        ),
    }
}

#[cfg(test)]
#[path = "testes_nome_cenario.rs"]
mod testes_nome_cenario;
