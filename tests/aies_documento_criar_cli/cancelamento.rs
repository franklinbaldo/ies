//! Uma CLI cria, a outra cancela a mesma identidade no mesmo SQLite.
use super::*;

/// Regressão técnica do BDD-273-02: o documento nascido pela CLI 246
/// precisa continuar sendo a mesma identidade cancelável no anti-IES.
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
    let (saidas, trilha, quantidade) = executar(&[&criar, &cancelar, &consultar], false);
    for (indice, saida) in saidas.iter().enumerate() {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "passo {indice}: stdout={} stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let resultado = json(&saidas[1]);
    assert_eq!(resultado["numero"], "1000002");
    assert_eq!(resultado["motivo"], motivo);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["confirmado"], true);
    let arvore = json(&saidas[2]);
    assert_eq!(arvore["documentos"].as_array().unwrap().len(), 2);
    assert_eq!(
        quantidade, 2,
        "cancelamento não elimina documento nem cria outro"
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
        "não repetir cancelamento após confirmação"
    );
}
