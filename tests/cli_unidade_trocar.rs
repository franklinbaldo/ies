//! Superfície de `ies unidade trocar SIGLA [--confirmar]` (#205): sigla
//! vazia sai com 2 antes de qualquer arquivo; sem configuração, 1
//! `fonte_invalida`, sem rede. Nenhum teste abre sessão nem acessa o SEI.
use std::process::{Command, Output};

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("IES deve iniciar")
}

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}

#[test]
fn sigla_vazia_e_erro_de_uso_antes_da_configuracao() {
    for confirmar in [false, true] {
        let mut args = vec!["unidade", "trocar", "  ", "--json"];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        let valor = json(&saida);
        assert_eq!(valor["codigo"], "entrada_invalida");
    }
}

#[test]
fn sem_configuracao_falha_fechado_sem_rede() {
    for confirmar in [false, true] {
        let mut args = vec!["unidade", "trocar", "PR-A", "--json"];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        assert_eq!(json(&saida)["codigo"], "fonte_invalida");
    }
}

#[test]
fn ajuda_diz_que_a_troca_vale_para_a_sessao_e_exige_sigla() {
    let ajuda = sem_sessao(&["unidade", "trocar", "--help"]);
    assert!(ajuda.status.success());
    assert!(String::from_utf8_lossy(&ajuda.stdout).contains("sessão"));
    let sem_sigla = sem_sessao(&["unidade", "trocar"]);
    assert_eq!(sem_sigla.status.code(), Some(2));
}
