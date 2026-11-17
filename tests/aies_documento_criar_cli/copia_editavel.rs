use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;

const CSS: &str = "<p style='text-align:justify'>&nbsp;</p>";
const ORIENTACAO: &str = "Edite o arquivo e, de preferência, siga os estilos que ele já contém.";

// Hipótese de minuta mínima para a transição no anti-IES. A revisão só surge
// com a inserção do novo documento, e a leitura posterior usa o mesmo SQLite.
pub(super) fn semear_revisao_na_criacao(banco: &Connection) {
    banco
        .execute_batch(&format!(
            "CREATE TEMP TRIGGER revisao_inicial AFTER INSERT ON documento_simulado
             BEGIN
                INSERT INTO documento_revisao
                    (origem, cenario, id_documento, versao, usuario, unidade, modificacao, html_corpo)
                VALUES (new.origem, new.cenario, new.id_documento, 1, 'TESTE', 'TESTE',
                        '17/11/2026 10:00:00', '{}');
             END;",
            CSS.replace('\'', "''")
        ))
        .unwrap();
}

#[test]
fn cria_base_e_copia_editavel_e_publica_orientacao_nas_duas_saidas() {
    for modo_json in [true, false] {
        let pasta = Pasta::nova();
        let mut args = criar(true);
        if !modo_json {
            args.remove(0);
        }
        let (saidas, trilha, quantidade) = executar_em(&pasta, &[&args], false, false);
        let saida = &saidas[0];
        assert!(
            saida.status.success(),
            "{}: {trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        assert_eq!(quantidade, 2);
        let nome = "1000002.v2.html";
        if modo_json {
            let resultado = json(saida);
            assert_eq!(resultado["copia_editavel"]["arquivo"], nome);
            assert_eq!(resultado["copia_editavel"]["orientacao"], ORIENTACAO);
            assert_eq!(resultado["requisicoes"], trilha.len());
        } else {
            let texto = String::from_utf8_lossy(&saida.stdout);
            assert!(texto.contains(&format!("Cópia editável: {nome}\n{ORIENTACAO}")));
        }
        let base = pasta.0.join("1000002.v1.html");
        let edicao = pasta.0.join(nome);
        let original = fs::read_to_string(&base).unwrap();
        assert!(original.contains(CSS));
        assert_eq!(fs::read_to_string(&edicao).unwrap(), original);
        assert_eq!(
            fs::metadata(&edicao).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::write(&edicao, "texto independente").unwrap();
        assert_eq!(fs::read_to_string(&base).unwrap(), original);
    }
}

#[test]
fn conflito_local_preserva_o_arquivo_e_o_numero_da_criacao_confirmada() {
    let pasta = Pasta::nova();
    let arquivo = pasta.privado("1000002.v2.html", "edição preexistente");
    let (saidas, trilha, quantidade) = executar_em(&pasta, &[&criar(true)], false, false);
    let saida = &saidas[0];
    assert_eq!(saida.status.code(), Some(1));
    let resultado = json(saida);
    assert_eq!(resultado["numero_sei"], "1000002");
    assert_eq!(resultado["confirmado"], true);
    assert!(resultado.get("copia_editavel").is_none());
    assert!(String::from_utf8_lossy(&saida.stderr).contains("A cópia editável falhou"));
    assert_eq!(fs::read_to_string(arquivo).unwrap(), "edição preexistente");
    assert!(!pasta.0.join("1000002.v1.html").exists());
    assert_eq!(quantidade, 2);
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_gerar:"))
            .count(),
        1
    );
}
