//! Superfície de `ies processo concluir NUP [--confirmar]` (Refs #172):
//! NUP inválido sai com 2 antes de qualquer arquivo; sem configuração, 1
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

/// Falha fechada (#695): saída 1, stdout vazio, stderr `Error: <Erro>`.
fn sem_arquivos_de_sessao(saida: &Output) {
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty(), "falha não publica stdout");
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.starts_with("Error: ArquivoPrivadoAusenteOuInacessivel"),
        "{erro}"
    );
}

#[test]
fn nup_invalido_e_erro_de_uso_antes_da_configuracao() {
    for confirmar in [false, true] {
        let mut args = vec!["processo", "concluir", " ", "--json"];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        assert!(
            saida.stdout.is_empty(),
            "uso inválido é recusado pelo clap, sem JSON"
        );
    }
}

#[test]
fn sem_configuracao_falha_fechado_sem_rede() {
    for confirmar in [false, true] {
        let mut args = vec!["processo", "concluir", "00001.000001/2026-01", "--json"];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        sem_arquivos_de_sessao(&saida);
    }
}

#[test]
fn opcao_desconhecida_e_erro_de_uso_do_clap() {
    let saida = sem_sessao(&["processo", "concluir", "00001.000001/2026-01", "--agendar"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}
