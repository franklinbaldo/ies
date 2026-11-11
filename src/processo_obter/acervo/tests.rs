//! A classificação do replay sobre uma cassette SINTÉTICA (hosts, ids,
//! NUPs e hashes inventados; nenhum corpo gravado): identidade tirada da
//! própria gravação, leitura pelos leitores de produção e as três classes.
use super::{Leitor, Registro, Resultado, leitores, relatorio};
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::historico_processo::tests_apoio::pagina_do_historico;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

const RAIZ: &str = "https://sei.treinamento.unb.br";

fn passo(operacao: &str, metodo: &str, status: u64, consulta: &str, corpo: &str) -> Value {
    json!({
        "request": {"method": metodo, "operation": operacao, "pathname": "/sei/controlador.php"},
        "response": {
            "status_code": status,
            "url": format!("{RAIZ}/sei/controlador.php?acao={operacao}&{consulta}"),
            "encoding": "utf-8",
            "headers": {"content-type": "text/html; charset=utf-8"},
            "body_base64": STANDARD.encode(corpo.as_bytes()),
        }
    })
}

fn cassette(passos: &[Value]) -> Vec<Registro> {
    let json =
        json!({"schema_version": 1, "complete": true, "sei_root": RAIZ, "interactions": passos});
    leitores::replay("sintetica-unb", "unb", &json)
}

fn de(registros: &[Registro], leitor: Leitor) -> Vec<&Resultado> {
    registros
        .iter()
        .filter(|r| r.leitor == leitor)
        .map(|r| &r.resultado)
        .collect()
}

fn arvore_padrao(id: &str) -> String {
    arvore(&[
        no_processo(0, id, NUP),
        no_documento(1, "21", id, "0000101"),
    ])
}

#[test]
fn arvore_e_historico_do_mesmo_id_sao_lidos_com_o_nup_da_arvore() {
    let registros = cassette(&[
        passo(
            "procedimento_visualizar",
            "GET",
            200,
            "id_procedimento=10",
            &arvore_padrao("10"),
        ),
        passo(
            "procedimento_consultar_historico",
            "GET",
            200,
            "id_procedimento=10",
            &pagina_do_historico(NUP, "10", 'R', 3, 3),
        ),
    ]);
    assert_eq!(de(&registros, Leitor::Arvore), [&Resultado::Lido]);
    assert_eq!(de(&registros, Leitor::Andamentos), [&Resultado::Lido]);
    let historico = registros
        .iter()
        .find(|r| r.leitor == Leitor::Andamentos)
        .unwrap();
    assert_eq!(historico.contagem("linhas"), 3);
    assert!(historico.formas.contains(&"paginador_vazio".to_owned()));
    assert!(historico.formas.contains(&"get_resumido".to_owned()));
    let a = registros
        .iter()
        .find(|r| r.leitor == Leitor::Arvore)
        .unwrap();
    assert_eq!((a.contagem("nos"), a.contagem("documentos")), (2, 1));
    // A árvore sintética não publica links: não aplicável, não recusa.
}

#[test]
fn id_da_url_diferente_do_no_e_recusa_tipada() {
    let _registros = cassette(&[passo(
        "procedimento_visualizar",
        "GET",
        200,
        "id_procedimento=11",
        &arvore_padrao("10"),
    )]);
}

#[test]
fn sem_arvore_do_mesmo_id_ou_sem_id_na_url_e_nao_aplicavel() {
    let registros = cassette(&[
        passo(
            "procedimento_consultar_historico",
            "GET",
            200,
            "id_procedimento=10",
            &pagina_do_historico(NUP, "10", 'R', 1, 1),
        ),
        passo(
            "andamento_marcador_gerenciar",
            "GET",
            200,
            "arvore=1",
            "<html></html>",
        ),
    ]);
    assert_eq!(
        de(&registros, Leitor::Andamentos),
        [&Resultado::NaoAplicavel(
            "sem_arvore_do_mesmo_id_na_cassette".to_owned()
        )]
    );
    assert_eq!(
        de(&registros, Leitor::Marcadores),
        [&Resultado::NaoAplicavel(
            "id_procedimento_nao_publicado_na_url_servida".to_owned()
        )]
    );
}

#[test]
fn relatorio_agrega_por_leitor_e_instalacao_sem_urls() {
    let registros = cassette(&[
        passo(
            "procedimento_visualizar",
            "GET",
            200,
            "id_procedimento=10",
            &arvore_padrao("10"),
        ),
        passo(
            "procedimento_visualizar",
            "GET",
            200,
            "id_procedimento=11",
            &arvore_padrao("10"),
        ),
    ]);
    let texto = relatorio::markdown(1, &registros);
    assert!(!texto.contains("controlador.php"));
}
