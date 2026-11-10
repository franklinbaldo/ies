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
fn entrada_vazia_sai_2_antes_da_sessao() {
    for args in [
        vec!["processo", "relacionar", " ", "--com", NUP, "--json"],
        vec!["processo", "relacionar", NUP, "--com", " ", "--json"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
    }
}
#[test]
fn sem_configuracao_previa_recusa_sem_rede() {
    let saida = sem_sessao(&["processo", "relacionar", NUP, "--com", OUTRO, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    sem_arquivos_de_sessao(&saida);
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
                assert_eq!(saida.stdout, b"");
                let erro = String::from_utf8_lossy(&saida.stderr);
                assert!(
                    erro.starts_with("Error: RelacionamentoConsigoMesmo"),
                    "{erro}"
                );
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
