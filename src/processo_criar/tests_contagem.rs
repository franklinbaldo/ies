//! #245: contagem real das chamadas inclusive quando o POST ficou inconclusivo.
use super::tests::{cadastro, lista, pedido, tela_controle};
use super::tests_candidatos::{
    arvore_com_propriedades, gerados_com_candidato, propriedades_correspondentes,
};
use super::*;
use crate::processo_concluir::tests_apoio::{ambiente, ok, rodar_com, trabalhar};
use crate::transporte::FalhaTransporte;

#[test]
fn falha_do_post_com_candidato_observado_preserva_orcamento_real() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
    ];
    let mut contador = 0;
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
            &mut contador,
        )
    });
    assert_eq!(resultado.unwrap_err().saida(), 3);
    assert_eq!(requests.len(), 8);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
    assert_eq!(contador, 8); // RED: antes, o erro descartava a contagem.
}

#[test]
fn recusa_antes_do_post_preserva_get_real() {
    let amb = ambiente();
    let mut invalido = pedido(true);
    invalido.unidade = Some("Unidade que nao e a da sessao");
    let mut contador = 0;
    let (resultado, requests) = rodar_com(&amb, vec![ok(tela_controle())], |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &invalido,
            rede,
            &mut contador,
        )
    });
    assert!(resultado.is_err());
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        0
    );
    assert_eq!(contador, 1);
}

#[test]
fn previa_sem_confirmar_expoe_a_mesma_contagem_que_o_transporte() {
    let amb = ambiente();
    let mut contador = 0;
    let respostas = vec![ok(tela_controle()), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(false),
            rede,
            &mut contador,
        )
    });
    assert!(!resultado.unwrap().confirmado);
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        0
    );
    assert_eq!(contador, 3);
}

/// A captura de `procedimento_gerar.html` no Todos @323eb01 publica
/// `maxlength=100` em `txtDescricao`. O limite vem da tela, não do IES.
fn cadastro_com_maxlength(limite: &str) -> String {
    cadastro().replace(
        "name='txtDescricao' value=''",
        &format!("name='txtDescricao' maxlength='{limite}' value=''"),
    )
}

#[test]
fn previa_recusa_especificacao_acima_do_maxlength_observado() {
    let amb = ambiente();
    let longa = "A".repeat(101);
    let mut solicitado = pedido(false);
    solicitado.especificacao = &longa;
    let mut contador = 0;
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro_com_maxlength("100")),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &solicitado,
            rede,
            &mut contador,
        )
    });
    let erro = resultado.expect_err("prévia deve respeitar maxlength da tela");
    assert_eq!(erro.subcodigo(), "especificacao_acima_do_limite");
    assert_eq!(erro.saida(), 1);
    assert_eq!(contador, 3);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

#[test]
fn confirmacao_recusa_especificacao_longa_antes_de_post() {
    let amb = ambiente();
    let longa = "A".repeat(101);
    let mut solicitado = pedido(true);
    solicitado.especificacao = &longa;
    let mut contador = 0;
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro_com_maxlength("100")),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &solicitado,
            rede,
            &mut contador,
        )
    });
    let erro = resultado.expect_err("maxlength deve impedir a escrita");
    assert_eq!(erro.subcodigo(), "especificacao_acima_do_limite");
    assert_eq!(erro.saida(), 1);
    assert_eq!(contador, 3);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

#[test]
fn comprimento_html_de_especificacao_considera_unidades_utf16() {
    let amb = ambiente();
    let longa = "🦉".repeat(51); // 51 escalares; 102 unidades UTF-16
    let mut solicitado = pedido(false);
    solicitado.especificacao = &longa;
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro_com_maxlength("100")),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &solicitado,
            rede,
        )
    });
    assert_eq!(
        resultado
            .expect_err("maxlength HTML considera UTF-16")
            .subcodigo(),
        "especificacao_acima_do_limite"
    );
    assert_eq!(requests.len(), 3);
}

#[test]
fn limite_exato_e_cadastro_sem_maxlength_nao_inventam_recusa() {
    let amb = ambiente();
    let exata = "A".repeat(100);
    let mut solicitado = pedido(false);
    solicitado.especificacao = &exata;
    for cadastro_html in [cadastro_com_maxlength("100"), cadastro()] {
        let respostas = vec![ok(tela_controle()), ok(lista(false)), ok(cadastro_html)];
        let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
            criar(
                sessao.origem,
                sessao.url_do_controle,
                sessao.cookie,
                &solicitado,
                rede,
            )
        });
        assert!(!resultado.unwrap().confirmado);
        assert_eq!(requests.len(), 3);
    }
}

// #245: a consulta AJAX de interessados é anterior ao POST criador.
// A falha HTTP observada não pode virar "HTML não reconhecido".
#[test]
fn ajax_de_interessado_preserva_http_503_e_contagem_sem_criar() {
    let amb = ambiente();
    let interessados = ["Pessoa A".to_owned()];
    let solicitado = PedidoCriar {
        interessados: &interessados,
        ..pedido(true)
    };
    let cadastro_com_interessado = format!(
        "{}\nobjAutoCompletarInteressadoRI1225 = new infraAjaxAutoCompletar('hdnIdInteressadoProcedimento','txtInteressadoProcedimento','controlador_ajax.php?acao_ajax=contato_auto_completar_contexto_RI1225&infra_hash={}');",
        cadastro().replace(
            "</form>",
            "<input type='hidden' name='hdnInteressadosProcedimento' value=''>\
             <input type='text' name='txtInteressadoProcedimento' value=''>\
             <select name='selInteressadosProcedimento'></select></form>"
        ),
        super::tests::HASH,
    );
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro_com_interessado),
        Err(FalhaTransporte::StatusHttpDeErroDoServidor(
            crate::transporte::StatusHttpRecusado::sem_mensagem(503),
        )),
    ];
    let mut contador = 0;
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar_com_contagem(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &solicitado,
            rede,
            &mut contador,
        )
    });
    let erro = resultado.expect_err("falha AJAX impede POST criador");
    assert_eq!(erro.subcodigo(), "status_http_de_erro_do_servidor");
    assert_eq!(erro.status_http().map(|http| http.status), Some(503));
    assert_eq!(erro.saida(), 1);
    assert_eq!(contador, 4);
    assert_eq!(requests.len(), 4);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(
        requests[3]
            .url
            .contains("acao_ajax=contato_auto_completar_contexto_RI1225")
    );
    assert!(
        requests
            .iter()
            .all(|r| { r.metodo != "POST" || !r.url.contains("acao=procedimento_gerar") })
    );
}
