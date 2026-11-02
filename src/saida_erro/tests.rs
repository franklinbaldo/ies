//! RED/GREEN da saída da falha `fonte_invalida` com recusa HTTP.
use super::*;
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
