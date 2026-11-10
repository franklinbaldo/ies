//! Superfície de `ies processo enviar NUP --para SIGLA [--manter-aberto]
//! [--confirmar]` (Refs #173): uso inválido sai com 2 antes de qualquer
//! arquivo; sem configuração, 1 `fonte_invalida`, sem rede. Nenhum teste
//! abre sessão nem acessa o SEI.
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

/// Falha fechada (#695): saída 1, stdout vazio, stderr `Error: <Variante>`.
fn sem_arquivos_de_sessao(saida: &Output) {
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty(), "falha não publica stdout");
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.starts_with("Error: ArquivoPrivadoAusenteOuInacessivel"),
        "{erro}"
    );
}

const NUP: &str = "00001.000001/2026-01";

#[test]
fn uso_invalido_sai_com_2_antes_da_configuracao() {
    for (args, _subcodigo) in [
        (
            vec!["processo", "enviar", " ", "--para", "TESTE", "--json"],
            "vazio",
        ),
        (
            vec!["processo", "enviar", NUP, "--para", " ", "--json"],
            "sigla_de_destino_invalida",
        ),
        (
            vec!["processo", "enviar", NUP, "--para", "A\tB", "--json"],
            "sigla_de_destino_invalida",
        ),
        (
            vec![
                "processo", "enviar", NUP, "--para", "A", "--para", "B", "--json",
            ],
            "mais_de_um_destino_nao_suportado",
        ),
    ] {
        for confirmar in [false, true] {
            let mut args = args.clone();
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
}

#[test]
fn sem_configuracao_falha_fechado_sem_rede() {
    for extra in [&[][..], &["--confirmar"][..], &["--manter-aberto"][..]] {
        let mut args = vec!["processo", "enviar", NUP, "--para", "TESTE", "--json"];
        args.extend_from_slice(extra);
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        sem_arquivos_de_sessao(&saida);
    }
}

#[test]
fn sem_para_e_opcao_desconhecida_sao_erro_do_clap() {
    for args in [
        vec!["processo", "enviar", NUP],
        vec!["processo", "enviar", NUP, "--para", "X", "--retorno"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        assert_eq!(saida.stdout, b"");
    }
}

#[test]
fn bdd_remover_anotacoes_e_opcao_da_cli_sem_confirmacao_cega() {
    // Dado um processo e a opção observada chkSinRemoverAnotacoes;
    // Quando peço prévia, a gramática aceita a flag (sem sessão, não há GET);
    // Então o erro é de fonte, e não argumento desconhecido.
    let base = [
        "processo",
        "enviar",
        NUP,
        "--para",
        "TESTE",
        "--remover-anotacoes",
        "--json",
    ];
    let previa = sem_sessao(&base);
    assert_eq!(previa.status.code(), Some(1));
    sem_arquivos_de_sessao(&previa);
    // Sem sessão o comando não pode observar nem confirmar a tela.
    // A recusa continua sem rede, mas agora a confirmação requer preflight.
    let mut confirmar = base.to_vec();
    confirmar.push("--confirmar");
    let saida = sem_sessao(&confirmar);
    assert_eq!(saida.status.code(), Some(1));
    sem_arquivos_de_sessao(&saida);
    // Sem trava de ensaio: a confirmação segue para a sessão, que falta.
}
