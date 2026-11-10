//! Testes de ciência documental: prova inerte e recusa na segunda ciência.
//! Referência: Todos `tests/test_documento_ciente_010.py` @323eb01.
use super::*;

/// Regressão #271: <noscript> depende do modo de scripts do navegador.
/// O conteúdo nele não prova que a Lista de Ciências relida contém a
/// atividade nova, mesmo que o parser indexe o checkbox.
#[test]
fn lista_em_noscript_nao_prova_ciencia_documental() {
    let html = lista(ATIVIDADE)
        .replacen(
            "<table id='tblAndamentos'",
            "<noscript><table id='tblAndamentos'",
            1,
        )
        .replacen("</table>", "</table></noscript>", 1);
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(aceite(ATIVIDADE)),
        ok(arvore_com_lista()),
        ok(html),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(erro, E::NaoConfirmado { requisicoes: 7, .. }));
    assert_eq!(pedidos.len(), 7);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1,
        "sem retry da ciência"
    );
    assert_eq!(pedidos.last(), Some(&get(&absoluta(&link_da_lista()))));
}

/// Caso do Todos `tests/test_documento_ciente_010.py` @323eb01:
/// no documento a segunda ciência é recusada SEM limite diário. A
/// validação de Infra 200 é preservada, mas não comprova escrita nova.
/// Não se repete o GET potencialmente mutador.
#[test]
fn segunda_ciencia_documental_e_validacao_sem_retry() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(
            "<html><body><textarea id='txaInfraValidacao'>Usuário já deu ciência neste documento.</textarea></body></html>"
                .to_owned(),
        ),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    let E::NaoConfirmado {
        falha_do_aceite: Some(super::super::documento::FalhaDoAceite::ValidacaoDaInfra(recusa)),
        ..
    } = &erro
    else {
        panic!("{erro:?}");
    };
    assert_eq!(
        Some(recusa.as_str()),
        Some("Usuário já deu ciência neste documento.")
    );
    assert!(matches!(
        erro,
        E::NaoConfirmado {
            falha_do_aceite: Some(super::super::documento::FalhaDoAceite::ValidacaoDaInfra(_)),
            requisicoes: 6,
            ..
        }
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}
