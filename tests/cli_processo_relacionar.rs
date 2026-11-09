//! Entrada e trava de escrita de `ies processo relacionar`.
//! Não abre login nem acessa SEI. Testa a ausência de efeitos remotos.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";
const OUTRO: &str = "00001.000002/2026-01";
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
fn entrada_vazia_sai_2_antes_da_sessao() {
    for args in [
        vec!["processo", "relacionar", " ", "--com", NUP, "--json"],
        vec!["processo", "relacionar", NUP, "--com", " ", "--json"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
        assert_eq!(json(&saida)["subcodigo"], "numero_processo_vazio");
    }
}
#[test]
fn sem_configuracao_previa_recusa_sem_rede() {
    let saida = sem_sessao(&["processo", "relacionar", NUP, "--com", OUTRO, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["codigo"], "fonte_invalida");
}
/// BDD-258-05: autorrelacionamento é inválido por identidade, sem sessão
/// e sem rede; confirmar ou remover não podem adiar essa recusa.
#[test]
fn bdd_258_05_autorrelacionamento_recusado_antes_da_sessao() {
    for destino in [NUP, "00001000001202601"] {
        for remover in [false, true] {
            for confirmar in [false, true] {
                let mut args = vec!["processo", "relacionar", NUP, "--com", destino, "--json"];
                if remover {
                    args.push("--remover");
                }
                if confirmar {
                    args.push("--confirmar");
                }
                let saida = sem_sessao(&args);
                assert_eq!(saida.status.code(), Some(1), "stderr={:?}", saida.stderr);
                let valor = json(&saida);
                assert_eq!(valor["subcodigo"], "relacionamento_consigo_mesmo");
                assert_eq!(valor["retentavel"], false);
            }
        }
    }
}

#[test]
fn confirmar_e_remover_confirmar_sem_sessao_recusam_sem_escrita() {
    for remover in [false, true] {
        let mut args = vec![
            "processo",
            "relacionar",
            NUP,
            "--com",
            OUTRO,
            "--confirmar",
            "--json",
        ];
        if remover {
            args.push("--remover");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1));
        assert_eq!(
            json(&saida)["subcodigo"],
            "arquivo_privado_ausente_ou_inacessivel"
        );
    }
}
#[test]
fn destino_obrigatorio_e_flag_desconhecida_saem_2() {
    for args in [
        vec!["processo", "relacionar", NUP],
        vec!["processo", "relacionar", NUP, "--com", NUP, "--rest"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
    }
}

/// Regressão técnica da trava WIP (AGENTS.md); não é um BDD novo.
#[test]
fn escrita_wip_com_sessao_sintetica_recusa_antes_da_rede() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let pasta = std::env::temp_dir().join(format!(
        "ies-relacionar-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();
    let origem = pasta.join("origem");
    let url = pasta.join("url");
    let cookie = pasta.join("cookie");
    for (arquivo, texto) in [
        (&origem, "https://sei.invalid/sei".to_owned()),
        (
            &url,
            format!(
                "https://sei.invalid/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
                "a".repeat(64)
            ),
        ),
        (&cookie, "PHPSESSID=SINTETICO".to_owned()),
    ] {
        fs::write(arquivo, texto).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    for remover in [false, true] {
        let mut args = vec![
            "processo",
            "relacionar",
            NUP,
            "--com",
            "00001.000002/2026-01",
            "--confirmar",
            "--json",
        ];
        if remover {
            args.push("--remover");
        }
        let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(args)
            .env_remove("IES_CONFIG_FILE")
            .env("IES_URL_CONTROLE_FILE", &url)
            .env("IES_COOKIE_FILE", &cookie)
            .env("IES_ORIGEM_FILE", &origem)
            .env("IES_INDICE_FILE", pasta.join("indice.sqlite3"))
            .output()
            .expect("IES deve iniciar");
        assert_eq!(saida.status.code(), Some(1));
        assert_eq!(json(&saida)["subcodigo"], "origem_fora_do_treinamento");
    }
    fs::remove_dir_all(pasta).unwrap();
}
