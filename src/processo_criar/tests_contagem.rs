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
