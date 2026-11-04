//! Saída JSON e humana de `processo concluir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_falha, json_de_sucesso};
use crate::arvore_processo::MotivoAbertoEm;
use crate::processo_concluir::{
    ErroConcluir, FalhaDoEnvio, FormaDaConclusao, ProcessoConcluido, VerificacaoPorLeitura,
};
use serde_json::json;

fn previa() -> ProcessoConcluido {
    ProcessoConcluido {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        confirmado: false,
        alterado: false,
        forma: Some(FormaDaConclusao::Tela),
        aberto_em: vec!["TESTE".into(), "TESTE_1_1".into()],
        depois: None,
        reconciliado_por: None,
        requisicoes: 3,
    }
}

#[test]
fn previa_publica_o_contrato_sem_url() {
    let valor = json_de_sucesso(&previa());
    assert_eq!(
        valor,
        json!({"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
               "origem": "web", "confirmado": false, "alterado": false, "forma": "tela",
               "aberto_em": ["TESTE", "TESTE_1_1"], "depois_esperado": ["TESTE_1_1"],
               "depois": null, "reconciliado_por_leitura": false, "requisicoes": 3})
    );
    let texto = humano(&previa());
    assert!(texto.ends_with("Requisições: 3"), "{texto}");
    assert!(texto.contains("Para concluir, repita com --confirmar"));
}

#[test]
fn reconciliado_traz_a_falha_do_envio() {
    let feito = ProcessoConcluido {
        confirmado: true,
        alterado: true,
        depois: Some(vec!["TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
        requisicoes: 6,
        ..previa()
    };
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["reconciliado_por_leitura"], true);
    assert_eq!(
        valor["falha_do_envio"],
        json!({"subcodigo": "redirecionamento_nao_reconhecido_apos_concluir", "status_http": 302})
    );
}

#[test]
fn efeito_incerto_sai_com_tres_e_a_verificacao() {
    let erro = ErroConcluir::EfeitoIncertoNaConclusao {
        aberto_em: vec!["TESTE".into()],
        falha_do_envio: None,
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
    assert_eq!(erro.saida(), 3);
    let valor = json_de_falha(&erro);
    assert_eq!(valor["codigo"], "efeito_incerto");
    assert_eq!(valor["subcodigo"], "efeito_incerto_na_conclusao");
    assert_eq!(valor["aberto_em"], json!(["TESTE"]));
    assert_eq!(valor["verificacao"], "html_da_raiz_nao_publicado");
    let nao = ErroConcluir::ConclusaoNaoAplicada {
        aberto_em: vec!["TESTE".into()],
        depois: vec!["TESTE".into()],
        falha_do_envio: None,
    };
    assert_eq!((nao.saida(), nao.codigo()), (1, "escrita_nao_aplicada"));
}
