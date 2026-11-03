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

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}

#[test]
fn ajuda_mostra_termo_e_pagina() {
    let saida = ies(&["processo", "buscar", "--help"], false);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert!(texto.contains("<TERMO>"), "{texto}");
    assert!(texto.contains("--pagina <N>"), "{texto}");
    assert!(!texto.contains("--assunto"), "{texto}");
}

#[test]
fn termo_com_controle_e_erro_de_uso_antes_da_configuracao() {
    let saida = ies(&["--json", "processo", "buscar", "000\n01"], false);
    assert_eq!(saida.status.code(), Some(2));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "entrada_invalida");
    assert_eq!(valor["subcodigo"], "numero_processo_controle");
}

#[test]
fn opcoes_de_leitura_do_processo_nao_combinam_com_buscar() {
    let saida = ies(&["--json", "processo", "--resumo", "buscar", "1"], false);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "opcoes_incompativeis_com_busca");
}

#[test]
fn sem_configuracao_a_falha_e_tipada_e_sem_rede() {
    let saida = ies(&["--json", "processo", "buscar", "00001"], false);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "caminho_url_controle_nao_informado"
    );
}

#[test]
fn arquivos_de_sessao_ausentes_sao_falha_tipada() {
    let saida = ies(
        &["--json", "processo", "buscar", "00001", "--pagina", "2"],
        true,
    );
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "arquivo_privado_ausente_ou_inacessivel"
    );
}

#[test]
fn pagina_zero_e_erro_de_uso() {
    let saida = ies(&["processo", "buscar", "1", "--pagina", "0"], true);
    assert_eq!(saida.status.code(), Some(2));
    assert!(saida.stdout.is_empty());
}
