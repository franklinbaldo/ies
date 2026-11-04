//! Superfície de `ies documento buscar TERMO [--pagina N]` (Refs #61): erros
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

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}

#[test]
fn ajuda_mostra_termo_e_pagina_sem_assunto() {
    let saida = ies(&["documento", "buscar", "--help"], false);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert!(texto.contains("<TERMO>"), "{texto}");
    assert!(texto.contains("--pagina <N>"), "{texto}");
    assert!(!texto.contains("--assunto"), "{texto}");
}

#[test]
fn termo_com_controle_e_erro_de_uso_antes_da_configuracao() {
    let saida = ies(&["--json", "documento", "buscar", "000\t1"], false);
    assert_eq!(saida.status.code(), Some(2));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "entrada_invalida");
    assert_eq!(valor["subcodigo"], "termo_de_documento_contem_controle");
}

#[test]
fn termo_vazio_e_erro_de_uso() {
    let saida = ies(&["--json", "documento", "buscar", "  "], false);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "termo_de_documento_vazio");
}

#[test]
fn opcoes_de_leitura_do_documento_nao_combinam_com_buscar() {
    let saida = ies(
        &["--json", "documento", "--processo", "1", "buscar", "1"],
        false,
    );
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "opcoes_incompativeis_com_busca");
}

#[test]
fn sem_configuracao_a_falha_e_tipada_e_sem_rede() {
    let saida = ies(&["--json", "documento", "buscar", "0001"], false);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "caminho_url_controle_nao_informado"
    );
}

#[test]
fn arquivos_de_sessao_ausentes_sao_falha_tipada() {
    let saida = ies(
        &["--json", "documento", "buscar", "0001", "--pagina", "2"],
        true,
    );
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "arquivo_privado_ausente_ou_inacessivel"
    );
}
