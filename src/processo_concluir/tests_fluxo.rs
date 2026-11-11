//! RED/GREEN do fluxo de `processo concluir`: sequências exatas de
//! requisições (orçamento do contrato), decisão antes de escrever, uma
//! escrita nas duas formas e a tabela de reconciliação. Telas sintéticas.
use super::erro::{ErroConcluir, FalhaDoEnvio, VerificacaoPorLeitura};
use super::tests_apoio::*;
use super::{DesfechoDaConclusao, FormaDaConclusao, ProcessoConcluido};
use crate::arvore_processo::{ErroFuncaoDeConcluir, MotivoAbertoEm};
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

/// Os pedidos da escrita pela tela: GET da tela, POST, GET da verificação.
fn pela_tela_e_verificacao() -> Vec<Pedido> {
    pedidos(vec![
        get(&links().0),
        post(&absoluta(&action_de_concluir()), CORPO_DA_CONCLUSAO),
        get(&url_arvore()),
    ])
}

/// Os pedidos da escrita pela função: tela do nó, GET da função, verificação.
fn pela_funcao_e_verificacao() -> Vec<Pedido> {
    pedidos(vec![get(&links().1), get(&links().2), get(&url_arvore())])
}

fn com_desfecho(
    desfecho: DesfechoDaConclusao,
    aberto_em: &[&str],
    requisicoes: u8,
) -> ProcessoConcluido {
    ProcessoConcluido {
        nup: NUP.into(),
        id_procedimento: ID.into(),
        unidade: UNIDADE.into(),
        aberto_em: siglas(aberto_em),
        requisicoes,
        desfecho,
    }
}

fn previa(forma: FormaDaConclusao, aberto_em: &[&str]) -> ProcessoConcluido {
    com_desfecho(DesfechoDaConclusao::Previa { forma }, aberto_em, 3)
}

fn nao_aberto(confirmado: bool, aberto_em: &[&str]) -> ProcessoConcluido {
    com_desfecho(
        DesfechoDaConclusao::NaoAbertoNaUnidade { confirmado },
        aberto_em,
        3,
    )
}

fn concluido(forma: FormaDaConclusao, aberto_em: &[&str], depois: &[&str]) -> ProcessoConcluido {
    com_desfecho(
        DesfechoDaConclusao::Concluido {
            forma,
            depois: siglas(depois),
            reconciliado_por: None,
        },
        aberto_em,
        6,
    )
}

#[test]
fn previa_pela_tela_so_abre_o_processo() {
    let amb = ambiente();
    let (r, feitos) = rodar(&amb, false, abertura(arvore(&[UNIDADE], &tela())));
    let feito = r.unwrap();
    assert_eq!(feito, previa(FormaDaConclusao::Tela, &[UNIDADE]));
    assert_eq!(feito.depois_esperado(), Vec::<String>::new());
    assert_eq!(feitos, pedidos_da_abertura());
}

#[test]
fn previa_pela_funcao_nao_pede_a_tela_do_no() {
    let amb = ambiente();
    let html = arvore(&["OUTRA", UNIDADE], &funcao());
    let (r, feitos) = rodar(&amb, false, abertura(html));
    let feito = r.unwrap();
    assert_eq!(feito, previa(FormaDaConclusao::Funcao, &["OUTRA", UNIDADE]));
    assert_eq!(feito.depois_esperado(), siglas(&["OUTRA"]));
    assert_eq!(feitos, pedidos_da_abertura());
}

#[test]
fn confirmar_pela_tela_um_post_e_verificacao_pela_arvore() {
    let amb = ambiente();
    let respostas = com(
        abertura(arvore(&[UNIDADE], &tela())),
        vec![
            ok(tela_de_conclusao(ID)),
            r302(&volta_a_arvore()),
            ok(arvore(&[], vazio())),
        ],
    );
    let (r, feitos) = rodar(&amb, true, respostas);
    assert_eq!(
        r.unwrap(),
        concluido(FormaDaConclusao::Tela, &[UNIDADE], &[])
    );
    assert_eq!(feitos, pela_tela_e_verificacao());
}

#[test]
fn indice_com_identidade_quente_nao_confirma_conclusao_sem_releitura_remota() {
    let amb = ambiente();
    let inst = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice.registrar_entrada_por_id(&inst, false);
    let respostas = com(
        abertura(arvore(&[UNIDADE], &tela())),
        vec![
            ok(tela_de_conclusao(ID)),
            r302(&volta_a_arvore()),
            ok(arvore(&[UNIDADE], &tela())),
        ],
    );
    let (resultado, pedidos) = rodar(&amb, true, respostas);
    assert!(resultado.is_err(), "cache não substitui situação remota");
    assert_eq!(pedidos, pela_tela_e_verificacao());
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn confirmar_pela_tela_com_location_http_no_mesmo_host_e_esperada() {
    // Issue #766 do Todos: o GO devolve o `Location` em `http://` no mesmo
    // host. Só dado: nada é seguido, e a verificação é a mesma.
    let amb = ambiente();
    let http = crate::arvore_processo::tests_concluir::absoluta(&volta_a_arvore())
        .replacen("https://", "http://", 1);
    let respostas = com(
        abertura(arvore(&[UNIDADE], &tela())),
        vec![
            ok(tela_de_conclusao(ID)),
            r302(&http),
            ok(arvore(&[], vazio())),
        ],
    );
    let (r, feitos) = rodar(&amb, true, respostas);
    assert_eq!(
        r.unwrap(),
        concluido(FormaDaConclusao::Tela, &[UNIDADE], &[])
    );
    assert_eq!(feitos, pela_tela_e_verificacao());
}

#[test]
fn confirmar_pela_tela_volta_ao_controle_tambem_e_esperada() {
    let amb = ambiente();
    let principal = volta_a_arvore().replace("acao=arvore_visualizar", "acao=principal");
    let respostas = com(
        abertura(arvore(&["OUTRA", UNIDADE], &tela())),
        vec![
            ok(tela_de_conclusao(ID)),
            r302(&principal),
            ok(arvore(&["OUTRA"], vazio())),
        ],
    );
    let (r, _) = rodar(&amb, true, respostas);
    assert_eq!(
        r.unwrap(),
        concluido(FormaDaConclusao::Tela, &["OUTRA", UNIDADE], &["OUTRA"])
    );
}

#[test]
fn confirmar_pela_funcao_o_get_da_funcao_e_a_escrita() {
    let amb = ambiente();
    let respostas = com(
        abertura(arvore(&[UNIDADE], &funcao())),
        vec![
            ok(tela_do_no(ID)),
            ok(resposta_da_funcao()),
            ok(arvore(&[], vazio())),
        ],
    );
    let (r, feitos) = rodar(&amb, true, respostas);
    assert_eq!(
        r.unwrap(),
        concluido(FormaDaConclusao::Funcao, &[UNIDADE], &[])
    );
    assert_eq!(feitos, pela_funcao_e_verificacao());
}

#[test]
fn ja_fechado_na_unidade_e_sem_acao_e_sucesso_sem_escrita() {
    for confirmar in [false, true] {
        let amb = ambiente();
        let (r, feitos) = rodar(&amb, confirmar, abertura(arvore(&["OUTRA"], vazio())));
        assert_eq!(r.unwrap(), nao_aberto(confirmar, &["OUTRA"]));
        assert_eq!(feitos, pedidos_da_abertura());
    }
}

#[test]
fn recusas_antes_de_escrever_so_custam_a_abertura() {
    let casos: Vec<(String, ErroConcluir)> = vec![
        (
            arvore(&["OUTRA"], &tela()),
            ErroConcluir::UnidadeDaSessaoAusenteDaSituacao {
                aberto_em: siglas(&["OUTRA"]),
            },
        ),
        (
            arvore(&[UNIDADE], vazio()),
            ErroConcluir::ConclusaoNaoPublicadaNaBarra {
                aberto_em: siglas(&[UNIDADE]),
            },
        ),
        (
            arvore_para_concluir(
                ID,
                NUP,
                &format!("Processo sobrestado na unidade {}.<br />", ancora(UNIDADE)),
                &tela(),
            ),
            ErroConcluir::SituacaoDoProcessoNaoLidaAntesDeConcluir {
                motivo: MotivoAbertoEm::RaizPublicaOSobrestamento,
            },
        ),
        (
            arvore(&[UNIDADE], &[tela()[0].clone(), ACAO_FUNCAO.to_owned()]),
            ErroConcluir::Barra(
                crate::arvore_processo::ErroConcluirNaBarra::FormasDeConcluirRepetidasNaBarra,
            ),
        ),
    ];
    for (html, erro) in casos {
        let amb = ambiente();
        let (r, feitos) = rodar(&amb, true, abertura(html));
        assert_eq!(r, Err(erro));
        assert_eq!(feitos, pedidos_da_abertura());
    }
}

#[test]
fn tela_recusada_ou_redirecionada_nao_envia_o_post() {
    let outro = tela_de_conclusao("11");
    let marcada = tela_de_conclusao(ID).replace(
        "class=\"infraCheckbox\"  tabindex",
        "class=\"infraCheckbox\" checked=\"checked\" tabindex",
    );
    let casos = vec![
        (
            ok(outro),
            ErroConcluir::TelaDeConclusao(
                super::erro::ErroTelaDeConclusao::TelaDeConclusaoDeOutroProcesso,
            ),
        ),
        (
            ok(marcada),
            ErroConcluir::TelaDeConclusao(
                super::erro::ErroTelaDeConclusao::TelaDeConclusaoComReaberturaPreenchida,
            ),
        ),
        (
            r302(&volta_a_arvore()),
            ErroConcluir::TelaDaEscritaRedirecionada,
        ),
    ];
    for (resposta, erro) in casos {
        let amb = ambiente();
        let respostas = com(abertura(arvore(&[UNIDADE], &tela())), vec![resposta]);
        let (r, feitos) = rodar(&amb, true, respostas);
        assert_eq!(r, Err(erro));
        assert_eq!(feitos, pedidos(vec![get(&links().0)]));
    }
}

#[test]
fn tela_do_no_sem_a_funcao_nao_escreve() {
    let amb = ambiente();
    let sem = tela_do_no(ID).replace("function concluirProcesso(){", "function outra(){");
    let respostas = com(abertura(arvore(&[UNIDADE], &funcao())), vec![ok(sem)]);
    let (r, feitos) = rodar(&amb, true, respostas);
    assert_eq!(
        r,
        Err(ErroConcluir::FuncaoDeConcluir(
            ErroFuncaoDeConcluir::FuncaoDeConcluirAusenteNaTelaDoNo
        ))
    );
    assert_eq!(feitos, pedidos(vec![get(&links().1)]));
}

/// A forma `tela` com `resposta` ao POST e `depois` na verificação.
fn tela_com(
    resposta: Resposta,
    depois: Resposta,
) -> (Result<ProcessoConcluido, ErroConcluir>, Vec<Pedido>) {
    let amb = ambiente();
    let respostas = com(
        abertura(arvore(&[UNIDADE], &tela())),
        vec![ok(tela_de_conclusao(ID)), resposta, depois],
    );
    rodar(&amb, true, respostas)
}

#[test]
fn resposta_nao_esperada_e_unidade_fechada_reconcilia_pela_leitura() {
    let (r, feitos) = tela_com(ok("<html></html>".into()), ok(arvore(&[], vazio())));
    let feito = r.unwrap();
    assert!(feito.alterado());
    assert_eq!(
        feito.reconciliado_por().cloned(),
        Some(FalhaDoEnvio::RespostaSemRedirecionamento)
    );
    assert_eq!(feitos, pela_tela_e_verificacao());
    let (r, _) = tela_com(Ok((503, String::new(), None)), ok(arvore(&[], vazio())));
    assert!(matches!(
        r.unwrap().reconciliado_por().cloned(),
        Some(FalhaDoEnvio::Envio(_))
    ));
}

#[test]
fn unidade_ainda_aberta_depois_e_nao_aplicada() {
    let (r, feitos) = tela_com(r302(&volta_a_arvore()), ok(arvore(&[UNIDADE], &tela())));
    assert_eq!(
        r,
        Err(ErroConcluir::ConclusaoNaoAplicada {
            aberto_em: siglas(&[UNIDADE]),
            depois: siglas(&[UNIDADE]),
            falha_do_envio: None,
        })
    );
    assert_eq!(feitos, pela_tela_e_verificacao());
    let fora = volta_a_arvore().replace("acao=arvore_visualizar", "acao=procedimento_controlar");
    let (r, _) = tela_com(r302(&fora), ok(arvore(&[UNIDADE], &tela())));
    assert_eq!(
        r,
        Err(ErroConcluir::ConclusaoNaoAplicada {
            aberto_em: siglas(&[UNIDADE]),
            depois: siglas(&[UNIDADE]),
            falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
        })
    );
}

#[test]
fn leitura_que_falha_ou_nao_le_a_situacao_e_nao_confirmado() {
    let (r, feitos) = tela_com(r302(&volta_a_arvore()), Err(FalhaTransporte::Rede));
    let Err(erro @ ErroConcluir::NaoConfirmadoNaConclusao { .. }) = r else {
        panic!("esperava efeito incerto: {r:?}");
    };
    let ErroConcluir::NaoConfirmadoNaConclusao { verificacao, .. } = &erro else {
        unreachable!()
    };
    assert_eq!(
        **verificacao,
        VerificacaoPorLeitura::LeituraFalhou(crate::processo_obter::ErroProcessoObter::Transporte(
            FalhaTransporte::Rede
        ))
    );
    assert_eq!(feitos, pela_tela_e_verificacao());
    let sobrestado = arvore_para_concluir(
        ID,
        NUP,
        &format!("Processo sobrestado na unidade {}.<br />", ancora(UNIDADE)),
        vazio(),
    );
    let (r, _) = tela_com(r302(&volta_a_arvore()), ok(sobrestado));
    assert_eq!(
        r,
        Err(ErroConcluir::NaoConfirmadoNaConclusao {
            aberto_em: siglas(&[UNIDADE]),
            falha_do_envio: None,
            verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
                MotivoAbertoEm::RaizPublicaOSobrestamento
            )),
        })
    );
}

#[test]
fn falha_no_get_da_funcao_nao_repete_e_reconcilia() {
    let amb = ambiente();
    let respostas = com(
        abertura(arvore(&[UNIDADE], &funcao())),
        vec![
            ok(tela_do_no(ID)),
            Err(FalhaTransporte::Rede),
            ok(arvore(&[], vazio())),
        ],
    );
    let (r, feitos) = rodar(&amb, true, respostas);
    assert_eq!(
        r.unwrap().reconciliado_por().cloned(),
        Some(FalhaDoEnvio::Transporte(FalhaTransporte::Rede))
    );
    assert_eq!(feitos, pela_funcao_e_verificacao());
}
