//! Regressões adversariais de JS não observado na oferta de exclusão.
//! Cenários sintéticos derivados da tela `UnB` de treinamento (#254).
use super::tests::lista_observada;
use super::*;

fn sobrescrita_anterior() -> String {
    lista_observada("831").replacen(
        "function acaoExcluir(id,desc)",
        "window.acaoExcluir = null; function acaoExcluir(id,desc)",
        1,
    )
}

fn sobrescrita_em_outro_script() -> String {
    lista_observada("831").replacen(
        "</script>",
        "</script><script>window.acaoExcluir = null;</script>",
        1,
    )
}

fn sobrescrita_computada_em_outro_script() -> String {
    lista_observada("831").replacen(
        "</script>",
        "</script><script>window['acao' + 'Excluir'] = null;</script>",
        1,
    )
}

#[test]
fn recusa_outro_script_classico_mesmo_sem_simbolo_literal() {
    let html = lista_observada("831").replacen(
        "</script>",
        "</script><script>window['outraPropriedade'] = 1;</script>",
        1,
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao_de_teste(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn script_de_dados_adicional_nao_redefine_funcao_global() {
    let html = lista_observada("831").replacen(
        "</script>",
        "</script><script type=\"application/json\">{\"acaoExcluir\":null}</script>",
        1,
    );
    assert!(conferir_oferta_exclusao(&instalacao_de_teste(), &html, "10", "831").is_ok());
}

#[test]
fn recusa_reatribuicao_antes_da_declaracao() {
    assert_eq!(
        conferir_oferta_exclusao(&instalacao_de_teste(), &sobrescrita_anterior(), "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn recusa_reatribuicao_computada_em_script_classico_posterior() {
    assert_eq!(
        conferir_oferta_exclusao(
            &instalacao_de_teste(),
            &sobrescrita_computada_em_outro_script(),
            "10",
            "831"
        ),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn recusa_reatribuicao_em_script_classico_posterior() {
    assert_eq!(
        conferir_oferta_exclusao(
            &instalacao_de_teste(),
            &sobrescrita_em_outro_script(),
            "10",
            "831"
        ),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

fn instalacao_de_teste() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

#[test]
fn previa_recusa_sobrescritas_sem_post() {
    use crate::processo_acompanhar::fluxo::{ErroPrevia, Pedido, previa};
    use crate::processo_concluir::tests_apoio::{
        UNIDADE, abertura, absoluta, ambiente, arvore, get, ok, pedidos_da_abertura, rodar_com,
    };
    use crate::propriedades_processo::tests_apoio::acao_da_barra;

    let gerenciar = format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento=10&infra_hash={}",
        "a".repeat(64)
    );
    for html in [
        sobrescrita_anterior(),
        sobrescrita_em_outro_script(),
        sobrescrita_computada_em_outro_script(),
    ] {
        let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_barra(&gerenciar)]));
        respostas.push(ok(html));
        let amb = ambiente();
        let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
            previa(
                sessao,
                &Pedido {
                    nup,
                    unidade: None,
                    grupo: Some("G"),
                    observacao: None,
                    deixar: true,
                    confirmar: false,
                },
                indice,
                roteiro,
            )
        });
        assert!(matches!(
            resultado,
            Err(ErroPrevia::Estrutura(ErroAcompanhar::ExclusaoNaoOferecida))
        ));
        let mut esperados = pedidos_da_abertura();
        esperados.push(get(&absoluta(&gerenciar)));
        assert_eq!(pedidos, esperados);
        assert_eq!(pedidos.len(), 4);
        assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
    }
}
