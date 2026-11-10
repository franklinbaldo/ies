//! Regressões do GET potencialmente mutador: causa tipada, status HTTP e JSON.
//! Cenários sintéticos #271, sem acesso ao SEI ou retry.
use super::*;

/// RED #271: o erro tipado do único GET de escrita do processo não pode ser
/// descartado enquanto a releitura independente tenta provar a atividade.
#[derive(Clone, Copy)]
enum FalhaEsperada {
    Http403,
    Infra200,
    Rede,
}

fn conferir_falha_do_aceite_sem_retry(resposta: Resposta, causa: FalhaEsperada) {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([ok(tela()), resposta, ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    match causa {
        FalhaEsperada::Http403 => {
            assert!(matches!(
                erro,
                E::NaoConfirmado {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::Transporte(
                        FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                            status: 401 | 403,
                            ..
                        })
                    )),
                    ..
                }
            ));
        }
        FalhaEsperada::Infra200 => {
            let E::NaoConfirmado {
                falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(recusa)),
                ..
            } = &erro
            else {
                panic!("{erro:?}");
            };
            assert_eq!(Some(recusa.as_str()), Some("Solicitacao recusada"));
            assert!(matches!(
                erro,
                E::NaoConfirmado {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(_)),
                    ..
                }
            ));
        }
        FalhaEsperada::Rede => {
            assert!(matches!(
                erro,
                E::NaoConfirmado {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::Transporte(FalhaTransporte::Rede)),
                    ..
                }
            ));
        }
    }
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "o GET potencialmente mutador não pode ser repetido"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn get_de_ciencia_http_403_preserva_a_recusa_tipificada() {
    conferir_falha_do_aceite_sem_retry(
        Ok((
            403,
            "<html><body>Solicitação recusada</body></html>".to_owned(),
            None,
        )),
        FalhaEsperada::Http403,
    );
}

#[test]
fn get_de_ciencia_infra_200_preserva_a_recusa_tipificada() {
    conferir_falha_do_aceite_sem_retry(
        ok(
            "<html><textarea id='txaInfraValidacao'>Solicitacao recusada</textarea></html>"
                .to_owned(),
        ),
        FalhaEsperada::Infra200,
    );
}

#[test]
fn get_de_ciencia_falha_de_rede_preserva_o_erro_tipificado() {
    conferir_falha_do_aceite_sem_retry(Err(FalhaTransporte::Rede), FalhaEsperada::Rede);
}

// RED #271: a recusa por validação Infra usa HTTP 200, mas a CLI precisa
// publicar status_http=200 no JSON, sem confundir com aceite bem-sucedido.
#[test]
fn json_de_recusa_infra_200_publica_status_http_e_nao_autoriza_retry() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(
            "<html><textarea id='txaInfraValidacao'>Solicitacao recusada</textarea></html>"
                .to_owned(),
        ),
        ok(arvore_com_ciencias()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("recusa HTTP 200 nao equivale a ciencia");
    let E::NaoConfirmado {
        falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(recusa)),
        requisicoes: 6,
        ..
    } = &erro
    else {
        panic!("{erro:?}");
    };
    assert_eq!(Some(recusa.as_str()), Some("Solicitacao recusada"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}
