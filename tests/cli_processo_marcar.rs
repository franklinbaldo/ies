//! Contrato da CLI #255: valida entradas sem configuração, sem rede nem sessão.
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
    serde_json::from_slice(&saida.stdout).expect("resposta JSON")
}
#[test]
fn marcador_invalido_sai_2_antes_de_ler_sessao() {
    for (rotulo, subcodigo) in [
        ("sem-separador", "rotulo_sem_separador"),
        ("x#Nome", "id_do_marcador_nao_decimal"),
        ("501# ", "nome_do_marcador_vazio"),
    ] {
        let saida = sem_sessao(&["processo", "marcar", "NUP", "--marcador", rotulo, "--json"]);
        assert_eq!(saida.status.code(), Some(2), "{rotulo}");
        assert_eq!(json(&saida)["subcodigo"], subcodigo);
    }
}
#[test]
fn marcador_valido_tenta_sessao_sem_enviar_escrita() {
    for confirmar in [false, true] {
        let mut args = vec![
            "processo",
            "marcar",
            "NUP",
            "--retirar",
            "501#Teste",
            "--json",
        ];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        let valor = json(&saida);
        assert_eq!(valor["codigo"], "fonte_invalida");
        assert_ne!(valor["subcodigo"], "nao_implementado");
    }
}
#[test]
fn clap_rejeita_sem_operacao_duas_operacoes_e_texto_na_retirada() {
    for args in [
        vec!["processo", "marcar", "NUP"],
        vec![
            "processo",
            "marcar",
            "NUP",
            "--marcador",
            "1#A",
            "--retirar",
            "1#A",
        ],
        vec![
            "processo",
            "marcar",
            "NUP",
            "--retirar",
            "1#A",
            "--texto",
            "X",
        ],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn texto_acima_do_limite_observado_sai_2_sem_abrir_sessao() {
    let longo = "A".repeat(251);
    let saida = sem_sessao(&[
        "processo",
        "marcar",
        "NUP",
        "--marcador",
        "501#A",
        "--texto",
        &longo,
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "texto_do_marcador_longo");
}
#[test]
fn texto_multilinha_ou_unicode_nao_cria_recusa_antes_da_tela() {
    for texto in ["Primeira linha\nSegunda linha", "Emoji 🚀"] {
        for confirmar in [false, true] {
            let mut args = vec![
                "processo",
                "marcar",
                "NUP",
                "--marcador",
                "501#A",
                "--texto",
                texto,
                "--json",
            ];
            if confirmar {
                args.push("--confirmar");
            }
            let saida = sem_sessao(&args);
            assert_eq!(saida.status.code(), Some(1));
            if confirmar {
                assert_eq!(json(&saida)["subcodigo"], "escrita_ainda_fechada");
                assert_eq!(json(&saida)["requisicoes"], 0);
            } else {
                assert_eq!(json(&saida)["codigo"], "fonte_invalida");
            }
        }
    }
}

#[test]
fn atribuicao_confirmada_continua_fechada_e_exibe_previa_segura() {
    let json_saida = sem_sessao(&[
        "processo", "marcar", "NUP", "--marcador", "501#Revisão",
        "--confirmar", "--json",
    ]);
    assert_eq!(json_saida.status.code(), Some(1));
    let resposta = json(&json_saida);
    assert_eq!(resposta["subcodigo"], "escrita_ainda_fechada");
    assert_eq!(resposta["requisicoes"], 0);
    assert!(
        resposta["erro"].as_str().unwrap()
            .contains("execute sem --confirmar para ver a prévia")
    );

    let humana = sem_sessao(&[
        "processo", "marcar", "NUP", "--marcador", "501#Revisão", "--confirmar",
    ]);
    assert_eq!(humana.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&humana.stderr)
            .contains("execute sem --confirmar para ver a prévia")
    );
}

#[test]
fn retirada_confirmada_exige_sessao_privada_antes_de_qualquer_rede() {
    let saida = sem_sessao(&[
        "processo", "marcar", "NUP", "--retirar", "501#Revisão",
        "--confirmar", "--json",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["subcodigo"], "arquivo_privado_ausente_ou_inacessivel");
}
