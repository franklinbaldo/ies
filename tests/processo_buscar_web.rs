//! Superfície de `ies processo buscar TERMO [--pagina N]` (Refs #58): erros
//! de uso saem com 2 antes de ler a configuração e, sem sessão, a falha é
//! tipada. Nenhum teste abre sessão nem acessa o SEI.
use std::process::{Command, Output};

fn ies(args: &[&str], com_arquivos: bool) -> Output {
    let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
    comando.args(args).env_remove("IES_CONFIG_FILE");
    if com_arquivos {
        comando
            .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
            .env("IES_COOKIE_FILE", "/nao/existe/cookie")
            .env("IES_ORIGEM_FILE", "/nao/existe/origem");
    } else {
        comando
            .env_remove("IES_URL_CONTROLE_FILE")
            .env_remove("IES_COOKIE_FILE")
            .env_remove("IES_ORIGEM_FILE");
    }
    comando.output().expect("IES deve iniciar")
}

#[test]
fn ajuda_mostra_termo_e_pagina() {
    let saida = ies(&["processo", "buscar", "--help"], false);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert!(texto.contains("<TERMO>"), "{texto}");
    assert!(texto.contains("--pagina <N>"), "{texto}");
    assert!(texto.contains("--assunto <TEXTO>"), "{texto}");
    assert!(texto.contains("--tipo <TIPO>"), "{texto}");
    assert!(texto.contains("--de <DD/MM/AAAA>"), "{texto}");
    assert!(texto.contains("--ate <DD/MM/AAAA>"), "{texto}");
}

#[test]
fn termo_com_controle_e_erro_de_uso_antes_da_configuracao() {
    let saida = ies(&["--json", "processo", "buscar", "000\n01"], false);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}

#[test]
fn opcoes_de_leitura_do_processo_nao_combinam_com_buscar() {
    let saida = ies(
        &["--json", "processo", "--sem-documentos", "buscar", "1"],
        false,
    );
    assert_eq!(saida.status.code(), Some(2));
}

#[test]
fn sem_configuracao_a_falha_e_tipada_e_sem_rede() {
    let saida = ies(&["--json", "processo", "buscar", "00001"], false);
    assert_eq!(saida.status.code(), Some(1));
}

#[test]
fn arquivos_de_sessao_ausentes_sao_falha_tipada() {
    let saida = ies(
        &["--json", "processo", "buscar", "00001", "--pagina", "2"],
        true,
    );
    assert_eq!(saida.status.code(), Some(1));
}

#[test]
fn pagina_zero_e_erro_de_uso() {
    let saida = ies(&["processo", "buscar", "1", "--pagina", "0"], true);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&saida.stdout), "");
}

#[test]
fn data_invalida_e_recusada_antes_da_configuracao() {
    let saida = ies(
        &["--json", "processo", "buscar", "0001", "--de", "31/02/2026"],
        false,
    );
    assert_eq!(saida.status.code(), Some(2));
}
#[test]
fn periodo_invertido_e_recusado_antes_da_configuracao() {
    let saida = ies(
        &[
            "--json",
            "processo",
            "buscar",
            "0001",
            "--de",
            "02/01/2026",
            "--ate",
            "01/01/2026",
        ],
        false,
    );
    assert_eq!(saida.status.code(), Some(2));
}
