//! Hipótese sintética, não replay de requisição original.
use super::*;

#[test]
fn previa_texto_padrao_consulta_id_sem_post_criador() {
    let ajax = format!(
        "controlador_ajax.php?acao_ajax=texto_padrao_auto_completar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "7".repeat(64)
    );
    let html = cadastro_com_script(&format!(
        "objAutoCompletarTextoPadrao = new infraAjaxAutoCompletar('hdnIdTextoPadrao','txtTextoPadrao','{ajax}');"
    ))
    .replace(
        "</form>",
        r#"<input type="radio" name="rdoTextoInicial" value="T">
        <input type="text" name="txtTextoPadrao">
        <input type="hidden" name="hdnIdTextoPadrao"></form>"#,
    );
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(html));
    respostas.push(a::ok(
        "<?xml version='1.0'?><itens><item id='007' descricao='Ofício'/></itens>".to_owned(),
    ));
    let origem = "https://sei.test/sei";
    for (_, html, _) in respostas.iter_mut().flatten() {
        *html = html.replace(a::ORIGEM, origem);
    }
    let (resultado, pedidos) = a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let controle = sessao.url_do_controle.replace(a::ORIGEM, origem);
        let sessao = crate::processo_obter::SessaoDoProcesso {
            url_do_controle: &controle,
            origem,
            cookie: sessao.cookie,
        };
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: None,
            nivel: None,
            texto_padrao: Some("Ofício"),
            modelo: None,
        };
        executar_criacao(&sessao, &pedido, indice, rede, false)
    });
    let previa = resultado.unwrap();
    assert!(matches!(previa.estado, EstadoCriacao::Previa));
    let texto = previa.texto_padrao.unwrap();
    assert_eq!(texto.id, "007");
    assert_eq!(texto.nome, "Ofício");
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(a::post(&url(&ajax), "palavras_pesquisa=Of%EDcio"));
    for pedido in &mut esperados {
        pedido.url = pedido.url.replace(a::ORIGEM, origem);
    }
    assert_eq!(pedidos, esperados);
}

#[test]
fn confirmar_texto_padrao_ainda_recusa_antes_da_rede() {
    let amb = a::ambiente();
    let (resultado, pedidos) = a::rodar_com(&amb, vec![], |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: None,
            nivel: Some(crate::dominio::NivelDeAcesso::Publico),
            texto_padrao: Some("Ofício"),
            modelo: None,
        };
        executar_criacao(sessao, &pedido, indice, rede, true)
    });
    assert!(matches!(
        resultado,
        Err(ErroDocumentoCriar::ConfirmacaoDeTextoPadraoAindaNaoDisponivel)
    ));
    assert_eq!(
        pedidos,
        [] as [crate::processo_concluir::tests_apoio::Pedido; 0]
    );
}

#[test]
fn previa_texto_padrao_recusa_origens_fora_do_treinamento_sem_rede() {
    for origem in [a::ORIGEM, "https://sei.test.exemplo/sei"] {
        let amb = a::ambiente();
        let (resultado, pedidos) = a::rodar_com(&amb, vec![], |sessao, nup, indice, rede| {
            let controle = sessao.url_do_controle.replace(a::ORIGEM, origem);
            let sessao = crate::processo_obter::SessaoDoProcesso {
                url_do_controle: &controle,
                origem,
                cookie: sessao.cookie,
            };
            let pedido = PedidoCriacao {
                nup,
                unidade: None,
                tipo: "Despacho",
                descricao: None,
                nivel: None,
                texto_padrao: Some("Ofício"),
                modelo: None,
            };
            executar_criacao(&sessao, &pedido, indice, rede, false)
        });
        assert!(matches!(
            resultado,
            Err(ErroDocumentoCriar::OrigemNaoAutorizada)
        ));
        assert!(pedidos.is_empty(), "{origem}");
    }
}
