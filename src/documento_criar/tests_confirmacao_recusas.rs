//! Recusas HTTP e da Infra no POST de criação: a causa sobrevive à
//! releitura, sem repetir a escrita (#246). Parte de `tests_confirmacao`.
use super::tests_confirmacao::{RedeSintetica, foto, submissao_sintetica};
use super::*;
use crate::formulario_web::RespostaDoPost;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado};
use std::io::Cursor;

#[test]
fn falha_http_403_do_post_preserva_recusa_apesar_de_arvore_concorrente() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        |rede, submissao, _| {
            rede.chamadas.push("POST criador");
            rede.submissoes.push(submissao);
            Err::<RespostaDoPost<Cursor<Vec<u8>>>, _>(FalhaTransporte::Status(StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("acesso negado sintético".to_owned()),
            }))
        },
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    let erro = resultado.expect_err("POST com falha não confirma a criação concorrente");
    assert_eq!(
        erro,
        ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::Status(StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("acesso negado sintético".into()),
            })
        ))
    );
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1, "nunca repetir POST");
}

#[test]
fn falha_http_503_do_post_preserva_mensagem_saneada() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        |rede, submissao, _| {
            rede.chamadas.push("POST criador");
            rede.submissoes.push(submissao);
            Err::<RespostaDoPost<Cursor<Vec<u8>>>, _>(FalhaTransporte::Status(StatusHttpRecusado {
                status: 503,
                mensagem_do_servidor: Some("indisponibilidade sintética".to_owned()),
            }))
        },
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    let erro = resultado.expect_err("HTTP 503 não confirma efeito");
    assert_eq!(
        erro,
        ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::Status(StatusHttpRecusado {
                status: 503,
                mensagem_do_servidor: Some("indisponibilidade sintética".into()),
            })
        ))
    );
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}

#[test]
fn redirecionamento_invalido_ou_id_alheio_nao_confirma() {
    for destino in [
        "controlador.php?acao=documento_visualizar&id_documento=999",
        "controlador.php?acao=procedimento_trabalhar&id_documento=102",
        "//sei/controlador.php?acao=documento_visualizar&id_documento=102",
    ] {
        let mut rede = RedeSintetica::default();
        let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
        let resultado = postar_e_verificar(
            &mut rede,
            submissao_sintetica(),
            &cookie,
            &foto(0),
            "Despacho",
            |rede, submissao, _| {
                rede.chamadas.push("POST criador");
                rede.submissoes.push(submissao);
                Ok(RespostaDoPost {
                    resposta: RespostaWeb {
                        status_code: 302,
                        mime: "text/html".to_owned(),
                        content_length: None,
                        localizacao: Some(destino.to_owned()),
                        corpo: Cursor::new(Vec::new()),
                    },
                    localizacao: Some(destino.to_owned()),
                })
            },
            |rede| {
                rede.chamadas.push("GET arvore");
                Ok(foto(1))
            },
        );
        assert!(matches!(
            resultado,
            Err(ErroConfirmacaoDaCriacao::NaoConfirmado)
        ));
        assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
        assert_eq!(rede.submissoes.len(), 1);
    }
}

#[test]
fn validacao_infra_200_preserva_a_causa_apos_post() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        |rede, submissao, _| {
            rede.chamadas.push("POST criador");
            rede.submissoes.push(submissao);
            Ok(RespostaDoPost {
                resposta: RespostaWeb {
                    status_code: 200,
                    mime: "text/html; charset=UTF-8".to_owned(),
                    content_length: None,
                    localizacao: None,
                    corpo: Cursor::new(
                        "<html><textarea id=\"txaInfraValidacao\">Recusa sintética</textarea></html>"
                            .as_bytes()
                            .to_vec(),
                    ),
                },
                localizacao: None,
            })
        },
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    let erro = resultado.expect_err("validacao Infra 200 e escrita incerta");
    assert_eq!(
        erro,
        ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(
            ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei("Recusa sintética".into())
        )
    );
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}
