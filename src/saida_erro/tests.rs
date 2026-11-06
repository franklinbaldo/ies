//! RED/GREEN da saída da falha `fonte_invalida` com recusa HTTP.
use super::*;
use serde_json::json;
use std::collections::BTreeSet;

fn chaves(valor: &Value) -> BTreeSet<&str> {
    valor
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn falha_local_mantem_exatamente_a_forma_anterior() {
    let valor = json_de_erro_fonte("cookie_invalido", "formato inválido", None);
    assert_eq!(
        valor,
        json!({
            "codigo": "fonte_invalida",
            "subcodigo": "cookie_invalido",
            "erro": "formato inválido",
            "retentavel": false
        })
    );
    assert_eq!(
        linha_humana("cookie_invalido", "formato inválido", None),
        "IES: formato inválido [cookie_invalido]"
    );
}

#[test]
fn recusa_http_publica_status_e_mensagem_do_servidor() {
    let http = StatusHttpRecusado {
        status: 500,
        mensagem_do_servidor: Some("Erro acessando o banco de dados.".to_owned()),
    };
    let valor = json_de_erro_fonte(
        "status_http_de_erro_do_servidor",
        "o SEI respondeu com erro do servidor (5xx)",
        Some(&http),
    );
    assert_eq!(
        valor,
        json!({
            "codigo": "fonte_invalida",
            "subcodigo": "status_http_de_erro_do_servidor",
            "erro": "o SEI respondeu com erro do servidor (5xx)",
            "retentavel": false,
            "status_http": 500,
            "mensagem_do_servidor": "Erro acessando o banco de dados."
        })
    );
    assert_eq!(
        linha_humana(
            "status_http_de_erro_do_servidor",
            "erro do servidor",
            Some(&http)
        ),
        "IES: erro do servidor [status_http_de_erro_do_servidor] \
         (HTTP 500: Erro acessando o banco de dados.)"
    );
}

#[test]
fn recusa_sem_mensagem_publica_so_o_status() {
    let http = StatusHttpRecusado::sem_mensagem(302);
    let valor = json_de_erro_fonte("redirecionamento_recusado", "redirecionamento", Some(&http));
    assert_eq!(valor["status_http"], 302);
    assert!(valor.get("mensagem_do_servidor").is_none());
    assert_eq!(
        chaves(&valor),
        BTreeSet::from(["codigo", "erro", "retentavel", "status_http", "subcodigo"])
    );
    assert_eq!(
        linha_humana("redirecionamento_recusado", "redirecionamento", Some(&http)),
        "IES: redirecionamento [redirecionamento_recusado] (HTTP 302)"
    );
}

#[test]
fn validacao_da_infra_com_200_publica_so_a_mensagem() {
    let http = StatusHttpRecusado {
        status: 200,
        mensagem_do_servidor: Some("Protocolo inválido.".to_owned()),
    };
    let valor = json_de_erro_fonte("validacao_recusada_pelo_sei", "validação", Some(&http));
    assert_eq!(valor["mensagem_do_servidor"], "Protocolo inválido.");
    assert!(valor.get("status_http").is_none());
    assert_eq!(
        linha_humana("validacao_recusada_pelo_sei", "validação", Some(&http)),
        "IES: validação [validacao_recusada_pelo_sei] (SEI: Protocolo inválido.)"
    );
}

#[test]
fn fonte_publica_serde_direto_preserva_bytes_compactos_do_contrato() {
    let servidor_500 = StatusHttpRecusado {
        status: 500,
        mensagem_do_servidor: Some("Falha saneada do servidor.".to_owned()),
    };
    let infra_200 = StatusHttpRecusado {
        status: 200,
        mensagem_do_servidor: Some("Validação recusada.".to_owned()),
    };
    let redirect = StatusHttpRecusado::sem_mensagem(302);
    let casos = [
        (
            "cookie_invalido",
            "cookie inválido",
            None,
            r#"{"codigo":"fonte_invalida","erro":"cookie inválido","retentavel":false,"subcodigo":"cookie_invalido"}"#,
        ),
        (
            "status_http_de_erro_do_servidor",
            "erro HTTP",
            Some(&servidor_500),
            r#"{"codigo":"fonte_invalida","erro":"erro HTTP","mensagem_do_servidor":"Falha saneada do servidor.","retentavel":false,"status_http":500,"subcodigo":"status_http_de_erro_do_servidor"}"#,
        ),
        (
            "validacao_recusada_pelo_sei",
            "validação",
            Some(&infra_200),
            r#"{"codigo":"fonte_invalida","erro":"validação","mensagem_do_servidor":"Validação recusada.","retentavel":false,"subcodigo":"validacao_recusada_pelo_sei"}"#,
        ),
        (
            "redirecionamento_recusado",
            "recusa 302",
            Some(&redirect),
            r#"{"codigo":"fonte_invalida","erro":"recusa 302","retentavel":false,"status_http":302,"subcodigo":"redirecionamento_recusado"}"#,
        ),
    ];
    for (subcodigo, mensagem, http, esperado) in casos {
        // Golden independente: também confere o JSON legado existente.
        let contrato = json_de_erro_fonte(subcodigo, mensagem, http).to_string();
        assert_eq!(contrato, esperado, "{subcodigo}");
        let mut bytes = Vec::new();
        serde_json::to_writer(&mut bytes, &falha_fonte_publica(subcodigo, mensagem, http)).unwrap();
        let emitido = String::from_utf8(bytes).unwrap();
        assert_eq!(emitido, esperado, "{subcodigo}");
        assert!(!emitido.contains("infra_hash"));
        assert!(!emitido.contains('\u{1b}'));
    }
}
