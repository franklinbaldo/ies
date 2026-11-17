//! Testes sintéticos da associação entre aceites e listas (#271).
use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::json;

fn resposta_html(operacao: &str, url: &str, html: &str) -> Value {
    json!({
        "request": {"operation": operacao, "method": "GET"},
        "response": {
            "status_code": 200,
            "url": url,
            "headers": {"content-type": "text/html"},
            "encoding": "utf-8",
            "body_base64": STANDARD.encode(html)
        }
    })
}

#[test]
fn replay_de_aceite_e_lista_usa_leitores_de_ambos_alvos() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let casos = [
        (
            "procedimento_ciencia",
            format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
            "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>",
        ),
        (
            "documento_ciencia",
            format!("{base}?acao=documento_ciencia&id_procedimento=10&id_documento=21"),
            "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>",
        ),
    ];
    for (operacao, url, aceite) in casos {
        let url_lista = if operacao == "documento_ciencia" {
            format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10&id_documento=21")
        } else {
            format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10")
        };
        let json = json!({
            "interactions": [
                resposta_html(operacao, &url, aceite),
                resposta_html("protocolo_ciencia_listar", &url_lista, lista)
            ]
        });
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 2);
        assert!(
            registros.iter().all(|r| r.resultado == Resultado::Lido),
            "{operacao}: {registros:?}"
        );
    }
}

#[test]
fn aceite_invalido_de_outro_alvo_nao_reutiliza_atividade_anterior() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let valido = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista_antiga = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({
        "interactions": [
            resposta_html(
                "procedimento_ciencia",
                &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                valido
            ),
            resposta_html(
                "procedimento_ciencia",
                &format!("{base}?acao=procedimento_ciencia&id_procedimento=11"),
                "<html><p>Sem atividade observada para o novo processo</p></html>"
            ),
            resposta_html(
                "protocolo_ciencia_listar",
                &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=11"),
                lista_antiga
            )
        ]
    });
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 3, "{registros:?}");
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert!(matches!(registros[1].resultado, Resultado::Recusado(_)));
    assert_eq!(
        registros[2].resultado,
        Resultado::NaoAplicavel("aceite_sem_atividade_observada".to_owned())
    );
}

#[test]
fn aceite_302_apos_aceite_valido_nao_autoriza_lista_anterior() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let valido = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista_antiga = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({
        "interactions": [
            resposta_html(
                "procedimento_ciencia",
                &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                valido
            ),
            {
                "request": {"operation": "documento_ciencia", "method": "GET"},
                "response": {"status_code": 302}
            },
            resposta_html(
                "protocolo_ciencia_listar",
                &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
                lista_antiga
            )
        ]
    });
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 3, "{registros:?}");
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("status_sem_prova_de_aceite".to_owned())
    );
    assert_eq!(
        registros[2].resultado,
        Resultado::NaoAplicavel("aceite_sem_atividade_observada".to_owned())
    );
}

#[test]
fn lista_de_outro_processo_nao_comprova_aceite_valido() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let html = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "procedimento_ciencia",
            &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
            html
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=11"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 2);
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("lista_de_outro_processo".to_owned())
    );
}

#[test]
fn aceite_sem_id_na_url_nao_herda_processo_de_outra_resposta() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let anterior = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let posterior = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=43#ID-43');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='43'></td></tr></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "procedimento_ciencia",
            &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
            anterior
        ),
        resposta_html(
            "procedimento_ciencia",
            &format!("{base}?acao=procedimento_ciencia"),
            posterior
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 3);
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("id_procedimento_nao_observado".to_owned())
    );
    assert_eq!(
        registros[2].resultado,
        Resultado::NaoAplicavel("aceite_sem_atividade_observada".to_owned())
    );
}

#[test]
fn resposta_302_nao_e_prova_de_aceite() {
    let json = json!({"interactions": [{
        "request": {"operation": "procedimento_ciencia", "method": "GET"},
        "response": {"status_code": 302}
    }]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 1);
    assert_eq!(
        registros[0].resultado,
        Resultado::NaoAplicavel("status_sem_prova_de_aceite".to_owned())
    );
}

#[test]
fn lista_sem_atividade_do_aceite_nao_e_sucesso() {
    let json = json!({"interactions": [{
        "request": {"operation": "protocolo_ciencia_listar", "method": "GET"},
        "response": {"status_code": 200}
    }]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 1);
    assert_ne!(registros[0].resultado, Resultado::Lido);
}

#[test]
fn lista_de_outro_documento_nao_comprova_aceite_documental() {
    // RED #271: o replay não pode perder a identidade do documento aceito.
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let aceite = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "documento_ciencia",
            &format!("{base}?acao=documento_ciencia&id_procedimento=10&id_documento=21"),
            aceite
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10&id_documento=22"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 2);
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("lista_de_outro_documento".to_owned())
    );
}

#[test]
fn lista_documental_nao_comprova_aceite_de_processo() {
    // RED #271: uma lista CD de documento não comprova ciência do processo.
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let aceite = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "procedimento_ciencia",
            &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
            aceite
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10&id_documento=21"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 2);
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("lista_documental_nao_prova_ciencia_do_processo".to_owned())
    );
}

#[test]
fn lista_documental_sem_id_nao_prova_aceite_documental() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let aceite = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "documento_ciencia",
            &format!("{base}?acao=documento_ciencia&id_procedimento=10&id_documento=21"),
            aceite
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 2);
    assert_eq!(registros[0].resultado, Resultado::Lido);
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("id_documento_da_lista_nao_observado".to_owned())
    );
}

/// RED #271: o rótulo da cassette não pode transformar uma abertura
/// `procedimento_visualizar` em GET de ciência. A prova depende da
/// ação da URL realmente servida, não só do campo `request.operation`.
#[test]
fn rotulo_de_ciencia_com_url_de_visualizacao_nao_comprova_processo() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let aceite = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><input type='checkbox' name='chkInfraItem1' value='42'></table></html>";
    let json = json!({"interactions": [
        resposta_html(
            "procedimento_ciencia",
            &format!("{base}?acao=procedimento_visualizar&id_procedimento=10"),
            aceite
        ),
        resposta_html(
            "protocolo_ciencia_listar",
            &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
            lista
        )
    ]});
    let registros = replay("unb", &json);
    assert_eq!(registros.len(), 2);
    assert_eq!(
        registros[0].resultado,
        Resultado::NaoAplicavel("acao_de_ciencia_nao_observada".to_owned())
    );
    assert_eq!(
        registros[1].resultado,
        Resultado::NaoAplicavel("aceite_sem_atividade_observada".to_owned())
    );
}

/// RED #271: no documento também não se aceita operação rotulada como
/// ciência quando `acao` da URL é outra ou aparece duplicada.
#[test]
fn rotulo_documental_nao_substitui_acao_unica_da_url_servida() {
    let base = "https://sei.treinamento.unb.br/sei/controlador.php";
    let aceite = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
    for consulta in [
        "acao=procedimento_visualizar&id_procedimento=10&id_documento=21",
        "acao=documento_ciencia&acao=procedimento_visualizar&id_procedimento=10&id_documento=21",
    ] {
        let json = json!({"interactions": [
            resposta_html("documento_ciencia", &format!("{base}?{consulta}"), aceite)
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_eq!(
            registros[0].resultado,
            Resultado::NaoAplicavel("acao_de_ciencia_nao_observada".to_owned()),
            "{consulta}"
        );
    }
}
