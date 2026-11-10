//! RED/GREEN do fluxo de `processo reabrir`: sequências exatas de
//! requisições (orçamento do contrato), decisão antes de escrever, a tela
//! do nó nas duas formas, uma escrita e a tabela de reconciliação. Telas
//! sintéticas (as de `processo concluir` mais a barra e a tela do nó de
//! reabrir).
use super::erro::ErroReabrir as R;
use super::{PedidoDaReabertura, ProcessoReaberto, reabrir_processo};
use crate::arvore_processo::tests_reabrir::{
    ACAO_REABRIR, link_de_reabrir, tela_do_no_pela_funcao, tela_do_no_pela_variavel,
};
use crate::arvore_processo::{ErroEnderecoDeReabrir, FormaDaReabertura, MotivoAbertoEm};
use crate::erro_html::ErroHtml;
use crate::processo_concluir::tests_apoio::*;
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura as V};
use crate::transporte::FalhaTransporte;

fn siglas(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|s| (*s).to_owned()).collect()
}

fn com(mut base: Vec<Resposta>, depois: Vec<Resposta>) -> Vec<Resposta> {
    base.extend(depois);
    base
}

fn pedidos(depois: Vec<Pedido>) -> Vec<Pedido> {
    let mut todos = pedidos_da_abertura();
    todos.extend(depois);
    todos
}

fn reabrir() -> Vec<String> {
    vec![ACAO_REABRIR.to_owned()]
}

fn rodar_reabrir(
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoReaberto, R>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        let pedido = PedidoDaReabertura {
            nup,
            unidade: None,
            confirmar,
        };
        reabrir_processo(sessao, &pedido, indice, roteiro)
    })
}

/// Tela do nó, GET da URL de reabrir, GET da verificação.
fn escrita_e_verificacao() -> Vec<Pedido> {
    pedidos(vec![
        get(&links().1),
        get(&absoluta(&link_de_reabrir(ID))),
        get(&url_arvore()),
    ])
}

fn previa(aberto_em: &[&str]) -> ProcessoReaberto {
    ProcessoReaberto {
        nup: NUP.into(),
        id_procedimento: ID.into(),
        unidade: UNIDADE.into(),
        confirmado: false,
        alterado: false,
        oferecido: true,
        forma: None,
        aberto_em: siglas(aberto_em),
        depois: None,
        reconciliado_por: None,
        requisicoes: 3,
    }
}

fn reaberto(forma: FormaDaReabertura, aberto_em: &[&str], depois: &[&str]) -> ProcessoReaberto {
    ProcessoReaberto {
        confirmado: true,
        alterado: true,
        forma: Some(forma),
        depois: Some(siglas(depois)),
        requisicoes: 6,
        ..previa(aberto_em)
    }
}

/// A página que o GET de reabrir devolveu nas gravações (molde da tela do nó).
fn resposta_da_reabertura() -> String {
    tela_do_no_pela_variavel(ID).replace(
        "<title>SEI - </title>",
        "<title>SEI - Reabrir Processo</title>",
    )
}

#[test]
fn previa_so_abre_o_processo() {
    for aberto_em in [&[][..], &["OUTRA"][..]] {
        let (r, feitos) = rodar_reabrir(false, abertura(arvore(aberto_em, &reabrir())));
        assert_eq!(r.unwrap(), previa(aberto_em));
        assert_eq!(feitos, pedidos_da_abertura());
    }
}

#[test]
fn ja_aberto_na_unidade_e_sucesso_sem_escrita() {
    for confirmar in [false, true] {
        let (r, feitos) = rodar_reabrir(confirmar, abertura(arvore(&[UNIDADE], &tela())));
        assert_eq!(
            r.unwrap(),
            ProcessoReaberto {
                confirmado: confirmar,
                oferecido: false,
                ..previa(&[UNIDADE])
            }
        );
        assert_eq!(feitos, pedidos_da_abertura());
    }
}

#[test]
fn confirmar_pela_variavel_da_tela_do_no() {
    let respostas = com(
        abertura(arvore(&[], &reabrir())),
        vec![
            ok(tela_do_no_pela_variavel(ID)),
            ok(resposta_da_reabertura()),
            ok(arvore(&[UNIDADE], &tela())),
        ],
    );
    let (r, feitos) = rodar_reabrir(true, respostas);
    assert_eq!(
        r.unwrap(),
        reaberto(FormaDaReabertura::Variavel, &[], &[UNIDADE])
    );
    assert_eq!(feitos, escrita_e_verificacao());
}

#[test]
fn confirmar_pela_funcao_da_tela_do_no() {
    let respostas = com(
        abertura(arvore(&["OUTRA"], &reabrir())),
        vec![
            ok(tela_do_no_pela_funcao(ID)),
            ok(resposta_da_reabertura()),
            ok(arvore(&["OUTRA", UNIDADE], &funcao())),
        ],
    );
    let (r, feitos) = rodar_reabrir(true, respostas);
    assert_eq!(
        r.unwrap(),
        reaberto(FormaDaReabertura::Funcao, &["OUTRA"], &["OUTRA", UNIDADE])
    );
    assert_eq!(feitos, escrita_e_verificacao());
}

#[test]
fn recusas_antes_de_escrever_so_custam_a_abertura() {
    let estranha = arvore_para_concluir(ID, NUP, "Situação nunca vista.", &reabrir());
    let repetida = arvore(&[], &[ACAO_REABRIR.to_owned(), ACAO_REABRIR.to_owned()]);
    let casos = [
        (
            estranha,
            R::SituacaoDoProcessoNaoLidaAntesDeReabrir {
                motivo: MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada,
            },
        ),
        (
            arvore(&[UNIDADE], &reabrir()),
            R::UnidadeDaSessaoJaNaSituacao {
                aberto_em: siglas(&[UNIDADE]),
            },
        ),
        (
            arvore(&["OUTRA"], vazio()),
            R::ReaberturaNaoPublicadaNaBarra {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            repetida,
            R::Barra(ErroHtml::repetido("funcoes_de_reabrir_repetidas_na_barra")),
        ),
    ];
    for (html, erro) in casos {
        for confirmar in [false, true] {
            let (r, feitos) = rodar_reabrir(confirmar, abertura(html.clone()));
            assert_eq!(r, Err(erro.clone()));
            assert_eq!(feitos, pedidos_da_abertura());
        }
    }
}

#[test]
fn tela_do_no_recusada_nao_escreve() {
    let sem = tela_do_no_pela_variavel(ID).replace("var linkReabrirProcesso", "var linkOutro");
    let casos = [
        (
            ok(sem),
            R::EnderecoDeReabrir(ErroEnderecoDeReabrir::Html(ErroHtml::ausente(
                "endereco_de_reabrir_ausente_na_tela_do_no",
            ))),
        ),
        (r302(&volta_a_arvore()), R::TelaDoNoRedirecionada),
        (
            Err(FalhaTransporte::Rede),
            R::TelaDoNo(FalhaTransporte::Rede),
        ),
    ];
    for (resposta, erro) in casos {
        let respostas = com(abertura(arvore(&[], &reabrir())), vec![resposta]);
        let (r, feitos) = rodar_reabrir(true, respostas);
        assert_eq!(r, Err(erro));
        assert_eq!(feitos, pedidos(vec![get(&links().1)]));
    }
}

/// `resposta` ao GET de reabrir e `depois` na verificação.
fn escrita_com(resposta: Resposta, depois: Resposta) -> (Result<ProcessoReaberto, R>, Vec<Pedido>) {
    let respostas = com(
        abertura(arvore(&[], &reabrir())),
        vec![ok(tela_do_no_pela_variavel(ID)), resposta, depois],
    );
    rodar_reabrir(true, respostas)
}

#[test]
fn resposta_nao_esperada_e_unidade_aberta_reconcilia_pela_leitura() {
    let casos = [
        (
            r302(&volta_a_arvore()),
            FalhaDoEnvio::RedirecionamentoNaReabertura,
        ),
        (
            Err(FalhaTransporte::Rede),
            FalhaDoEnvio::Transporte(FalhaTransporte::Rede),
        ),
    ];
    for (resposta, falha) in casos {
        let (r, feitos) = escrita_com(resposta, ok(arvore(&[UNIDADE], &tela())));
        let feito = r.unwrap();
        assert!(feito.alterado);
        assert_eq!(feito.reconciliado_por, Some(falha));
        assert_eq!(feitos, escrita_e_verificacao());
    }
}

#[test]
fn unidade_fora_depois_e_nao_aplicada() {
    let (r, feitos) = escrita_com(ok(resposta_da_reabertura()), ok(arvore(&[], &reabrir())));
    assert_eq!(
        r,
        Err(R::ReaberturaNaoAplicada {
            aberto_em: vec![],
            depois: vec![],
            falha_do_envio: None,
        })
    );
    assert_eq!(feitos, escrita_e_verificacao());
}

#[test]
fn leitura_que_falha_ou_nao_le_a_situacao_e_nao_confirmado() {
    let (r, feitos) = escrita_com(ok(resposta_da_reabertura()), Err(FalhaTransporte::Rede));
    let Err(erro @ R::NaoConfirmadoNaReabertura { .. }) = r else {
        panic!("esperava efeito incerto: {r:?}");
    };
    assert_eq!(erro.codigo(), "nao_confirmado");
    assert_eq!(
        erro,
        R::NaoConfirmadoNaReabertura {
            aberto_em: vec![],
            falha_do_envio: None,
            verificacao: Box::new(V::LeituraFalhou(
                crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Rede)
            )),
        }
    );
    assert_eq!(feitos, escrita_e_verificacao());
    let sobrestado = arvore_para_concluir(
        ID,
        NUP,
        &format!("Processo sobrestado na unidade {}.<br />", ancora(UNIDADE)),
        vazio(),
    );
    let (r, _) = escrita_com(r302(&volta_a_arvore()), ok(sobrestado));
    assert_eq!(
        r,
        Err(R::NaoConfirmadoNaReabertura {
            aberto_em: vec![],
            falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
            verificacao: Box::new(V::SituacaoNaoLida(
                MotivoAbertoEm::RaizPublicaOSobrestamento
            )),
        })
    );
}
