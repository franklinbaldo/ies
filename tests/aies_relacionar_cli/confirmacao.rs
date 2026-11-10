//! Confirmações da CLI IES contra anti-IES e SQLite no mesmo ensaio TLS local.
//! O servidor sintético não autoriza afirmar paridade com o SEI real.
use super::*;

fn saida_json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("saída JSON")
}

#[test]
fn bdd_258_02_adicionar_confirmado_por_cli_com_post_unico_e_releitura() {
    let (saida, eventos, vinculos) = executar_com(false, false, false, Modo::ConfirmarAdicao);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "saída: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = saida_json(&saida);
    assert_eq!(valor["confirmado"], true);
    assert_eq!(valor["alterado"], true);
    assert_eq!(valor["sem_efeito"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(vinculos, 1, "POST altera o SQLite compartilhado");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| *e == "POST procedimento_relacionar")
            .count(),
        1
    );
    let gravacao = eventos
        .iter()
        .position(|e| e == "POST procedimento_relacionar")
        .unwrap();
    assert_eq!(eventos[gravacao + 1], "GET procedimento_relacionar");
}

#[test]
fn bdd_258_03_remover_confirmado_por_cli_com_post_unico_e_releitura() {
    let (saida, eventos, vinculos) = executar_com(true, false, false, Modo::ConfirmarRemocao);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "saída: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = saida_json(&saida);
    assert_eq!(valor["confirmado"], true);
    assert_eq!(valor["remover"], true);
    assert_eq!(valor["sem_efeito"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(vinculos, 0, "remoção bilateral no SQLite");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| *e == "POST procedimento_excluir_relacionamento")
            .count(),
        1
    );
    let gravacao = eventos
        .iter()
        .position(|e| e == "POST procedimento_excluir_relacionamento")
        .unwrap();
    assert_eq!(eventos[gravacao + 1], "GET procedimento_relacionar");
}

#[test]
fn bdd_258_04_post_aceite_inconclusivo_nao_gera_sucesso_nem_retry() {
    // O anti-IES altera o banco, mas o proxy responde 200 genérico à CLI.
    // O estado final coincidente não prova aceite causal da confirmação.
    let (saida, eventos, vinculos) = executar_com(false, false, false, Modo::AdicaoSemAceite);
    assert_eq!(
        saida.status.code(),
        Some(1),
        "saída: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = saida_json(&saida);
    assert_eq!(valor["codigo"], "nao_confirmado");
    assert_eq!(valor["retentavel"], false);
    assert_eq!(
        vinculos, 1,
        "um POST pode ter alterado o servidor apesar do 200 inconclusivo"
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| *e == "POST procedimento_relacionar")
            .count(),
        1
    );
    assert!(
        !eventos
            .iter()
            .skip_while(|e| *e != "POST procedimento_relacionar")
            .skip(1)
            .any(|e| e == "POST procedimento_relacionar")
    );
}
