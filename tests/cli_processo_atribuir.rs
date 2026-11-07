//! Fronteira da CLI de atribuição (#264), sem login ou rede reais.
//! NUP inválido é recusado antes de ler a sessão; sessão ausente falha fechada.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";

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

#[test]
fn confirmar_com_referencia_legada_recusa_antes_da_sessao() {
    for operacao in [&["--usuario", "fulano"][..], &["--remover"][..]] {
        let mut args = vec!["processo", "atribuir", "sei://legado"];
        args.extend_from_slice(operacao);
        args.extend_from_slice(&["--confirmar", "--json"]);
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        let valor: serde_json::Value =
            serde_json::from_slice(&saida.stdout).expect("JSON na saída");
        assert_eq!(valor["codigo"], "entrada_invalida");
    }
}

#[test]
fn usuario_e_remover_juntos_sao_recusados_pelo_clap() {
    let saida = sem_sessao(&["processo", "atribuir", NUP, "--usuario", "x", "--remover"]);
    assert_eq!(saida.status.code(), Some(2));
}

#[test]
fn confirmar_valido_sem_sessao_retorna_fonte_invalida_sem_rede() {
    // A capacidade pública, quando habilitada, deve atravessar a validação
    // de NUP e falhar na leitura da sessão ausente, sem tocar na rede.
    let saida = sem_sessao(&[
        "processo",
        "atribuir",
        NUP,
        "--usuario",
        "aluno2",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).expect("JSON na saída");
    assert_eq!(valor["codigo"], "fonte_invalida");
}
