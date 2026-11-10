//! Uma CLI cria, a outra cancela a mesma identidade no mesmo SQLite.
use super::*;

/// Regressão técnica do BDD-273-02: a consulta independente da CLI
/// comprova o estado e o motivo; repetir o comando não altera o documento.
#[test]
fn bdd_273_02_criar_documento_e_cancelar_o_mesmo_id_com_releitura() {
    let criar = criar(true);
    let motivo = "  revisão necessária  ";
    let cancelar = [
        "--json",
        "documento",
        "--processo",
        NUP,
        "cancelar",
        "1000002",
        "--motivo",
        motivo,
        "--confirmar",
    ];
    let consultar = [
        "--json",
        "processo",
        NUP,
        "--sem-andamentos",
        "--sem-propriedades",
        "--sem-marcadores",
    ];
    let repetir = [
        "--json",
        "documento",
        "--processo",
        NUP,
        "cancelar",
        "1000002",
        "--motivo",
        "outro motivo",
        "--confirmar",
    ];
    // Os cinco comandos compartilham a mesma identidade e o mesmo SQLite.
    // A última consulta é posterior à recusa do segundo cancelamento.
    let (saidas, trilha, quantidade) = executar(
        &[&criar, &cancelar, &consultar, &repetir, &consultar],
        false,
    );
    for indice in [0, 1, 2, 4] {
        assert_eq!(
            saidas[indice].status.code(),
            Some(0),
            "passo {indice}: stdout={} stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saidas[indice].stdout),
            String::from_utf8_lossy(&saidas[indice].stderr)
        );
    }
    let resultado = json(&saidas[1]);
    assert_eq!(resultado["numero"], "1000002");
    assert_eq!(resultado["motivo"], motivo);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(resultado["reconciliado_por_leitura"], true);

    // Releitura pela CLI real, não pelo renderizador ou pelo banco diretamente.
    // O documento anterior permanece, e o cancelado conserva o mesmo ID.
    for indice in [2, 4] {
        let arvore = json(&saidas[indice]);
        let documentos = arvore["documentos"].as_array().expect("árvore da CLI");
        assert_eq!(documentos.len(), 2);
        let original = documentos
            .iter()
            .find(|d| d["numero_sei"] == "1000001")
            .expect("documento preexistente");
        assert_eq!(original["cancelado"], false);
        let cancelado = documentos
            .iter()
            .find(|d| d["numero_sei"] == "1000002")
            .expect("documento recém-criado");
        assert_eq!(cancelado["id_documento"], resultado["id_documento"]);
        assert_eq!(cancelado["icone"], "documento_cancelado");
        assert_eq!(cancelado["cancelado"], true);
        assert_eq!(cancelado["motivo_do_cancelamento"], motivo);
    }

    assert_eq!(
        saidas[3].status.code(),
        Some(1),
        "segunda tentativa: stdout={} stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saidas[3].stdout),
        String::from_utf8_lossy(&saidas[3].stderr)
    );
    assert_eq!(
        quantidade, 2,
        "cancelamento não elimina nem duplica documento"
    );
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_gerar:"))
            .count(),
        1
    );
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_cancelar:"))
            .count(),
        1,
        "a segunda tentativa não envia outro POST"
    );
}
