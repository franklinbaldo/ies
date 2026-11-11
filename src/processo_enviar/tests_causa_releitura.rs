//! Regressão técnica da ablação #718: a releitura pós-escrita preserva a causa.
//! Não é novo cenário BDD da CLI nem observação de cassette.
use super::tests_anotacoes::{anotacao, link_anotacao};
use super::tests_apoio::*;
use super::{PedidoDoEnvio, enviar_processo};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::transporte::FalhaTransporte;

#[test]
fn erro_da_releitura_de_anotacao_preserva_causa_sem_segundo_post() {
    // O destino foi confirmado por árvore, mas o GET independente não decide
    // a anotação por três motivos distintos: rede, redirect e outro processo.
    type Esperado = Box<dyn Fn(&super::ErroEnviar) -> bool>;
    let casos: [(_, Esperado); 3] = [
        (
            Err(FalhaTransporte::Rede),
            Box::new(|e: &super::ErroEnviar| {
                matches!(
                    e,
                    super::ErroEnviar::Anotacoes(
                        super::anotacoes::ErroDasAnotacoes::TelaDasAnotacoes(FalhaTransporte::Rede,)
                    )
                )
            }),
        ),
        (
            r302("controlador.php?acao=principal"),
            Box::new(|e: &super::ErroEnviar| {
                matches!(
                    e,
                    super::ErroEnviar::Anotacoes(
                        super::anotacoes::ErroDasAnotacoes::TelaDasAnotacoesRedirecionada,
                    )
                )
            }),
        ),
        (
            ok(anotacao("Outra", "999")),
            Box::new(|e: &super::ErroEnviar| {
                matches!(
                    e,
                    super::ErroEnviar::Anotacoes(
                        super::anotacoes::ErroDasAnotacoes::LeituraDasAnotacoes(
                            super::anotacoes::ErroLeituraDaAnotacao::AnotacaoDeOutroProcesso,
                        )
                    )
                )
            }),
        ),
    ];
    for (ultima_resposta, esperado) in casos {
        let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
        let respostas = com(
            abertura(arvore(&[UNIDADE], &acoes)),
            vec![
                ok(tela_de_envio(Versao::V40, ID)),
                ok(anotacao("Antes", ID)),
                ok(um_destino()),
                r302(&volta_a_arvore_do_envio()),
                ok(arvore(&[DESTINO], &[])),
                ultima_resposta,
            ],
        );
        let amb = ambiente();
        let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
            enviar_processo(
                sessao,
                &PedidoDoEnvio {
                    nup,
                    unidade: None,
                    para: DESTINO,
                    manter_aberto: false,
                    remover_anotacoes: true,
                    confirmar: true,
                },
                indice,
                rede,
            )
        });
        let erro = resultado.expect_err("anotação não comprovada não pode ser sucesso");
        assert!(
            std::error::Error::source(&erro).is_some(),
            "releitura pós-escrita precisa preservar sua fonte"
        );
        let super::ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes { causa, .. } = erro else {
            panic!("a releitura deve falhar sem confirmação e preservar a causa");
        };
        assert!(esperado(&causa), "{causa:?}");
        assert_eq!(pedidos.len(), 9);
        assert_eq!(
            pedidos
                .iter()
                .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_de_enviar()))
                .count(),
            1,
            "não repetir a escrita"
        );
    }
}
