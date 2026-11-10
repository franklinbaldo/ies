//! BDD-264-02/03: a própria CLI escreve uma única vez no anti-IES local,
//! depois um processo novo lê a mesma identidade. Sem acesso externo.
use super::*;

fn executar_e_reler(remover: bool, esperado: Option<&str>) {
    let (escrita, releitura, eventos, unidades, estado) =
        executar_cenario(true, remover, None, true);
    assert_eq!(
        escrita.status.code(),
        Some(0),
        "primeira CLI: {}",
        String::from_utf8_lossy(&escrita.stderr)
    );
    let resposta: serde_json::Value =
        serde_json::from_slice(&escrita.stdout).expect("JSON da escrita");
    assert_eq!(resposta["confirmado"], true);
    assert_eq!(resposta["alterado"], true);
    assert_eq!(resposta["usuario_atual"], "aluno1");
    assert_eq!(resposta["usuario_pedido"], serde_json::json!(esperado));
    // A confirmação reabre a árvore integralmente; as dez tentativas
    // pertencem à primeira CLI, antes da segunda consulta de prova.
    assert_eq!(resposta["requisicoes"], 10);

    let post_de_atribuicao = eventos
        .iter()
        .filter(|evento| evento.as_str() == "POST procedimento_atribuicao_cadastrar")
        .count();
    assert_eq!(post_de_atribuicao, 1, "não repetir escrita: {eventos:?}");
    assert_eq!(unidades, 1);
    let esperado_final = esperado.unwrap_or("");
    assert_eq!(estado, esperado_final);

    // Outro processo, nova invocação da CLI e nova leitura do estado Web.
    let segunda = releitura.expect("segunda consulta ies processo NUP");
    assert_eq!(
        segunda.status.code(),
        Some(0),
        "releitura: {}",
        String::from_utf8_lossy(&segunda.stderr)
    );
    let consulta: serde_json::Value =
        serde_json::from_slice(&segunda.stdout).expect("JSON da releitura");
    assert_eq!(consulta["numero"], NUP);
    let usuario = &consulta["aberto_em"][0]["atribuido_a"];
    if let Some(login) = esperado {
        assert_eq!(usuario["sigla"], login);
    } else {
        assert!(usuario.is_null(), "remoção ainda aparece: {consulta}");
    }
}

#[test]
fn bdd_264_02_confirmar_atribuicao_e_reler_pela_cli_real() {
    executar_e_reler(false, Some("aluno2"));
}

#[test]
fn bdd_264_03_confirmar_remocao_e_reler_pela_cli_real() {
    executar_e_reler(true, None);
}

#[test]
fn confirmacao_recusa_origem_fora_do_treinamento_sem_post_ou_rede() {
    let pasta = Pasta::nova();
    let origem = pasta.privado("origem", "https://sei.producao.exemplo/sei");
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let url = pasta.privado("url", &format!(
        "https://sei.producao.exemplo/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64),
    ));
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "processo",
            "atribuir",
            NUP,
            "--usuario",
            "aluno2",
            "--confirmar",
        ])
        .env("IES_URL_CONTROLE_FILE", url)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("HOME", &pasta.0)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap();
    falha(&saida, "OrigemForaDoTreinamento");
}

#[test]
fn bdd_264_03_resposta_403_apos_post_aplicado_nao_confirma_autoria() {
    let (escrita, releitura, eventos, _, estado) =
        executar_cenario_com_resposta(true, false, None, true, RespostaPost::Recusa403);
    falha(&escrita, "NaoConfirmadoNaAtribuicao");
    assert_eq!(
        eventos
            .iter()
            .filter(|evento| evento.as_str() == "POST procedimento_atribuicao_cadastrar")
            .count(),
        1,
        "nunca repetir o POST"
    );
    assert_eq!(estado, "aluno2", "a escrita realmente ocorreu no SQLite");
    let consulta = releitura.expect("segunda execução independente");
    assert_eq!(consulta.status.code(), Some(0));
    let novo: serde_json::Value = serde_json::from_slice(&consulta.stdout).unwrap();
    assert_eq!(novo["aberto_em"][0]["atribuido_a"]["sigla"], "aluno2");
}

#[test]
fn bdd_264_02_03_302_historico_informa_estado_relido_sem_falsa_confirmacao() {
    for (remover, esperado) in [(false, "aluno2"), (true, "")] {
        let (escrita, releitura, eventos, _, estado) =
            executar_cenario_com_resposta(false, remover, None, true, RespostaPost::Historico302);
        falha(&escrita, "NaoConfirmadoNaAtribuicao");
        let texto = String::from_utf8_lossy(&escrita.stderr);
        assert!(!texto.contains("Atribuição confirmada"));
        assert!(!texto.contains("Remoção confirmada"));
        assert_eq!(
            eventos
                .iter()
                .filter(|e| e.as_str() == "POST procedimento_atribuicao_cadastrar")
                .count(),
            1,
            "um único POST, sem retry: {eventos:?}"
        );
        assert_eq!(estado, esperado);
        let nova = releitura.expect("nova invocação ies processo NUP");
        assert_eq!(nova.status.code(), Some(0));
        let processo: serde_json::Value = serde_json::from_slice(&nova.stdout).unwrap();
        let atribuido = &processo["aberto_em"][0]["atribuido_a"];
        if esperado.is_empty() {
            assert!(atribuido.is_null());
        } else {
            assert_eq!(atribuido["sigla"], esperado);
        }
    }
}

#[test]
fn bdd_264_02_03_302_historico_preserva_erro_json_e_orcamento() {
    for (remover, esperado) in [(false, "aluno2"), (true, "")] {
        let (escrita, _, eventos, _, estado) =
            executar_cenario_com_resposta(true, remover, None, true, RespostaPost::Historico302);
        falha(&escrita, "NaoConfirmadoNaAtribuicao");
        assert_eq!(estado, esperado);
        assert_eq!(
            eventos
                .iter()
                .filter(|e| e.as_str() == "POST procedimento_atribuicao_cadastrar")
                .count(),
            1
        );
    }
}
