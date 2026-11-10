//! RED/GREEN do `--confirmar` de `processo enviar`: um POST, a leitura de
//! verificação e a tabela de reconciliação do contrato.
use super::ProcessoEnviado;
use super::erro::ErroEnviar as R;
use super::tests_apoio::*;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::tests_apoio::*;
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura as V};
use crate::transporte::FalhaTransporte;

/// Abertura, tela, consulta, resposta do POST e a árvore relida.
fn respostas(versao: Versao, resposta: Resposta, relida: Resposta) -> Vec<Resposta> {
    com(
        abertura(arvore(&[UNIDADE], &enviar())),
        vec![
            ok(tela_de_envio(versao, ID)),
            ok(um_destino()),
            resposta,
            relida,
        ],
    )
}

fn enviado(manter: bool, depois: &[&str]) -> ProcessoEnviado {
    ProcessoEnviado {
        confirmado: true,
        alterado: true,
        depois: Some(siglas(depois)),
        requisicoes: 7,
        ..previa(&[UNIDADE], manter)
    }
}

#[test]
fn confirmar_envia_uma_vez_e_verifica() {
    for versao in [Versao::V40, Versao::V41] {
        for (manter, depois) in [(false, &[DESTINO][..]), (true, &[UNIDADE, DESTINO][..])] {
            let (r, feitos) = rodar_envio(
                DESTINO,
                manter,
                true,
                respostas(
                    versao,
                    r302(&volta_a_arvore_do_envio()),
                    ok(arvore(depois, &[])),
                ),
            );
            assert_eq!(r.unwrap(), enviado(manter, depois));
            assert_eq!(feitos, pedidos_da_escrita(versao, manter));
        }
    }
}

#[test]
fn indice_aquecido_nao_autoriza_envio_sem_situacao_remota() {
    let amb = ambiente();
    let inst = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice.registrar_entrada_por_id(&inst, false);
    let roteiro = respostas(
        Versao::V41,
        r302(&volta_a_arvore_do_envio()),
        ok(arvore(&[UNIDADE], &[])),
    );
    let (resultado, pedidos) = rodar_com(&amb, roteiro, |sessao, nup, indice, envio| {
        let pedido = super::PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: false,
            confirmar: true,
        };
        super::enviar_processo(sessao, &pedido, indice, envio)
    });
    assert!(
        resultado.is_err(),
        "destino ausente na leitura não é sucesso"
    );
    assert_eq!(pedidos, pedidos_da_escrita(Versao::V41, false));
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn location_http_no_mesmo_host_e_a_resposta_esperada() {
    // Issue #766 do Todos (GO): `Location` em `http://` no host do POST.
    let http = crate::arvore_processo::tests_concluir::absoluta(&volta_a_arvore_do_envio())
        .replacen("https://", "http://", 1);
    let (r, feitos) = rodar_envio(
        DESTINO,
        true,
        true,
        respostas(
            Versao::V41,
            r302(&http),
            ok(arvore(&[UNIDADE, DESTINO], &[])),
        ),
    );
    assert_eq!(r.unwrap(), enviado(true, &[UNIDADE, DESTINO]));
    assert_eq!(feitos, pedidos_da_escrita(Versao::V41, true));
}

#[test]
fn resposta_nao_esperada_e_reconciliada_pela_leitura() {
    for (resposta, falha) in [
        (
            r302("controlador.php?acao=principal"),
            FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar,
        ),
        (
            ok("<html><body>ok</body></html>".into()),
            FalhaDoEnvio::RespostaDoEnvioSemRedirecionamento,
        ),
        (
            Err(FalhaTransporte::Rede),
            FalhaDoEnvio::Envio(ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)),
        ),
    ] {
        let (r, feitos) = rodar_envio(
            DESTINO,
            false,
            true,
            respostas(Versao::V41, resposta, ok(arvore(&[DESTINO], &[]))),
        );
        assert_eq!(
            r.unwrap(),
            ProcessoEnviado {
                reconciliado_por: Some(falha),
                ..enviado(false, &[DESTINO])
            }
        );
        assert_eq!(feitos, pedidos_da_escrita(Versao::V41, false));
    }
}

#[test]
fn leitura_sem_o_destino_e_escrita_nao_aplicada() {
    let (r, feitos) = rodar_envio(
        DESTINO,
        false,
        true,
        respostas(
            Versao::V41,
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[UNIDADE], &enviar())),
        ),
    );
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        R::EnvioNaoAplicado {
            aberto_em: siglas(&[UNIDADE]),
            depois: siglas(&[UNIDADE]),
            anotacao_depois: None,
            falha_do_envio: None,
        }
    );
    assert_eq!(erro.codigo(), "escrita_nao_aplicada");
    assert_eq!(feitos, pedidos_da_escrita(Versao::V41, false));
}

#[test]
fn origem_diferente_do_pedido_e_escrita_diferente() {
    for (manter, depois) in [(false, &[UNIDADE, DESTINO][..]), (true, &[DESTINO][..])] {
        let (r, _) = rodar_envio(
            DESTINO,
            manter,
            true,
            respostas(
                Versao::V40,
                r302(&volta_a_arvore_do_envio()),
                ok(arvore(depois, &[])),
            ),
        );
        let erro = r.unwrap_err();
        assert_eq!(
            erro,
            R::SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
                aberto_em: siglas(&[UNIDADE]),
                depois: siglas(depois),
                anotacao_depois: None,
                falha_do_envio: None,
            }
        );
        assert_eq!(erro.codigo(), "escrita_diferente_da_solicitada");
    }
}

#[test]
fn leitura_que_falha_e_nao_confirmado_sem_retry() {
    let (r, feitos) = rodar_envio(
        DESTINO,
        false,
        true,
        respostas(
            Versao::V41,
            Err(FalhaTransporte::Rede),
            Err(FalhaTransporte::Rede),
        ),
    );
    let erro = r.unwrap_err();
    assert!(
        matches!(
            &erro,
            R::NaoConfirmadoNoEnvio { verificacao, falha_do_envio: Some(_), .. }
                if matches!(**verificacao, V::LeituraFalhou(_))
        ),
        "{erro:?}"
    );
    assert_eq!(erro.codigo(), "nao_confirmado");
    assert_eq!(feitos, pedidos_da_escrita(Versao::V41, false));
}
