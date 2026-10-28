use std::process::Command;

fn ies(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn ajuda_raiz() {
    let output = ies(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("processo"));
}

#[test]
fn substantivo_sozinho_mostra_ajuda() {
    let output = ies(&["processo"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("enviar"));
}

#[test]
fn comando_sem_execucao_retorna_falha_estruturada() {
    let output = ies(&["documento", "77782513", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}

#[test]
fn uso_invalido_falha_antes_da_execucao() {
    let output = ies(&["processo", "enviar", "123"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn escrita_reconhecida_nao_executa_sei() {
    let output = ies(&[
        "processo",
        "enviar",
        "123",
        "--para",
        "PGE-PPI",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}

#[test]
fn leitura_de_processo_aceita_projecoes_sem_executar_sei() {
    let output = ies(&[
        "processo",
        "0064.001060/2023-08",
        "--sem-documentos",
        "--sem-marcadores",
        "--sem-andamentos",
        "--sem-propriedades",
        "--andamentos-completos",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
    assert!(
        value["dados"]["comando"]
            .as_str()
            .unwrap()
            .contains("sem_documentos: true")
    );
}

#[test]
fn leitura_de_lote_de_documentos_aceita_opcoes_globais() {
    let output = ies(&[
        "documento",
        "77782513",
        "77782514",
        "--processo",
        "0064.001060/2023-08",
        "--saida",
        "conteudos",
        "--com-propriedades",
        "--ocultar-inacessiveis",
        "--unidade",
        "PPI",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
    assert!(
        value["dados"]["comando"]
            .as_str()
            .unwrap()
            .contains("77782514")
    );
}

#[test]
fn unidade_listar_processos_aceita_saida() {
    let output = ies(&[
        "unidade",
        "listar-processos",
        "--saida",
        "processos.json",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
}

#[test]
fn busca_documento_permanece_subcomando() {
    let output = ies(&["documento", "buscar", "--assunto", "previdencia", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
}

#[test]
fn replay_falha_sem_acervo_e_nao_executa_o_sei() {
    let output = ies(&["replay", "processo-andamento-go", "--acervo", "inexistente", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "acervo_ausente");
}

#[test]
fn processo_nao_aceita_parametros_de_replay() {
    let output = ies(&["processo", "202600005002487", "--zip-file", "x.zip"]);
    assert_eq!(output.status.code(), Some(2));
}
