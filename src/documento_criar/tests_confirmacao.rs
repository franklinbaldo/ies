//! Ensaios sintéticos da fronteira real de escrita e verificação (#246).
//! Casos de Todos `src/todos/documentos.py::_conferir_criacao` @323eb01.
use super::*;
use crate::arvore_processo::{DocumentoNaArvore, NaturezaDoDocumento};
use crate::formulario_web::{CodificacaoDaTela, RespostaDoPost};
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado};
use std::io::Cursor;

#[derive(Default)]
struct RedeSintetica {
    chamadas: Vec<&'static str>,
    submissoes: Vec<crate::formulario_web::SubmissaoDerivada>,
}
fn documento(id: &str, numero: &str, tipo: &str) -> DocumentoNaArvore {
    DocumentoNaArvore {
        id: id.to_owned(),
        rotulo: format!("{tipo} {numero}"),
        numero_sei: Some(numero.to_owned()),
        pasta: None,
        natureza: NaturezaDoDocumento {
            icone: None,
            cancelado: None,
            motivo_do_cancelamento: None,
            dica: None,
        },
    }
}
fn foto(novos: usize) -> FotoDaCriacao {
    let mut documentos = vec![documento("101", "0000001", "Ofício")];
    if novos > 0 {
        documentos.push(documento("102", "0000002", "Despacho"));
    }
    if novos > 1 {
        documentos.push(documento("103", "0000003", "Despacho"));
    }
    FotoDaCriacao {
        id: "10".to_owned(),
        documentos,
        pastas_nao_carregadas: false,
    }
}
fn submissao_sintetica() -> crate::formulario_web::SubmissaoDerivada {
    let hash = "a".repeat(64);
    let html = format!(
        r#"<html><form id="frmDocumentoCadastro" method="post"
          action="controlador.php?acao=documento_gerar&amp;infra_hash={hash}">
          <input type="hidden" name="hdnIdSerie" value="291">
          <input type="hidden" name="hdnFlagDocumentoCadastro" value="1">
          <input type="radio" name="rdoNivelAcesso" id="optPublico" value="0">
          </form></html>"#
    );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_gerar&infra_hash={}",
        "b".repeat(64)
    );
    let servida = TelaServida {
        url: &url,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    derivar_submissao(
        &CADASTRAR,
        &instalacao,
        &servida,
        "UNIDADE TESTE",
        &[("hdnFlagDocumentoCadastro", "2"), ("rdoNivelAcesso", "0")],
    )
    .unwrap()
}
fn resposta_ok() -> RespostaDoPost<Cursor<Vec<u8>>> {
    // Destino sintético com ID conferível na fotografia posterior.
    let destino = "controlador.php?acao=documento_visualizar&id_documento=102".to_owned();
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: 302,
            mime: "text/html; charset=UTF-8".to_owned(),
            content_length: None,
            localizacao: Some(destino.clone()),
            corpo: Cursor::new(Vec::new()),
        },
        localizacao: Some(destino),
    }
}
fn escrever_ok(
    rede: &mut RedeSintetica,
    submissao: crate::formulario_web::SubmissaoDerivada,
    _cookie: &str,
) -> Result<RespostaDoPost<Cursor<Vec<u8>>>, FalhaTransporte> {
    rede.chamadas.push("POST criador");
    if submissao.efeito() != CADASTRAR.efeito {
        return Err(FalhaTransporte::UrlInvalida);
    }
    rede.submissoes.push(submissao);
    Ok(resposta_ok())
}
#[test]
fn confirma_com_um_post_e_uma_releitura_sem_retry() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let numero = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    assert!(matches!(numero, Ok(ref numero) if numero == "0000002"));
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}
#[test]
fn redirect_sem_location_nao_prova_a_criacao_mesmo_com_arvore_alterada() {
    // 302 sem Location não identifica a consequência do POST.
    // Uma mudança concorrente na árvore não deve legitimá-lo.
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
                    mime: "text/html; charset=UTF-8".to_owned(),
                    content_length: None,
                    localizacao: None,
                    corpo: Cursor::new(Vec::<u8>::new()),
                },
                localizacao: None,
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

#[test]
fn falha_do_post_ainda_rele_a_arvore_sem_repetir_escrita() {
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
            Err::<RespostaDoPost<Cursor<Vec<u8>>>, _>(FalhaTransporte::UrlInvalida)
        },
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(_))
    ));
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}
#[test]
fn dois_documentos_novos_sao_nao_confirmado_apos_post_unico() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(2))
        },
    );
    assert_eq!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::NaoDemonstradoNaArvore(
            ErroCriar::NaoConfirmado
        ))
    );
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}
#[test]
fn leitura_posterior_falha_sem_segunda_escrita() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &foto(0),
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Err(ErroProcessoObter::Transporte(FalhaTransporte::Rede))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::NaoConfirmadoNaReleitura(
            ErroProcessoObter::Transporte(FalhaTransporte::Rede)
        ))
    ));
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}

#[test]
fn foto_incompleta_recusa_antes_do_post() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let mut antes = foto(0);
    antes.pastas_nao_carregadas = true;
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &antes,
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::ArvoreIncompleta)
    ));
    assert!(
        rede.chamadas.is_empty(),
        "recusa antes de qualquer requisição"
    );
}

#[test]
fn foto_inicial_com_ids_duplicados_recusa_antes_do_post() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let mut antes = foto(0);
    antes.documentos.push(antes.documentos[0].clone());
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &antes,
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::FotoInicialInconsistente)
    ));
    assert!(rede.chamadas.is_empty(), "IDs ambíguos não autorizam POST");
}

#[test]
fn foto_inicial_sem_id_do_processo_recusa_antes_do_post() {
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let mut antes = foto(0);
    antes.id.clear();
    let resultado = postar_e_verificar(
        &mut rede,
        submissao_sintetica(),
        &cookie,
        &antes,
        "Despacho",
        escrever_ok,
        |rede| {
            rede.chamadas.push("GET arvore");
            Ok(foto(1))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroConfirmacaoDaCriacao::FotoInicialInconsistente)
    ));
    assert!(
        rede.chamadas.is_empty(),
        "identidade ausente não autoriza POST"
    );
}

#[test]
fn redirect_para_host_externo_nao_comprova_criacao() {
    // Um 302 não seguido para outro host NÃO vincula o documento novo
    // ao único POST. Mudança concorrente da árvore continua inconclusiva.
    let mut rede = RedeSintetica::default();
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let externo = "https://externo.exemplo.test/sei/controlador.php?acao=procedimento_trabalhar";
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
                    mime: "text/html; charset=UTF-8".to_owned(),
                    content_length: None,
                    localizacao: Some(externo.to_owned()),
                    corpo: Cursor::new(Vec::<u8>::new()),
                },
                localizacao: Some(externo.to_owned()),
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

#[test]
fn redirecionamento_302_local_com_arvore_comprovada() {
    for localizacao in [
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_visualizar&id_documento=102",
        "controlador.php?acao=documento_visualizar&id_documento=102",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=102",
    ] {
        let mut rede = RedeSintetica::default();
        let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
        let numero = postar_e_verificar(
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
                        mime: "text/html; charset=UTF-8".to_owned(),
                        content_length: None,
                        localizacao: Some(localizacao.to_owned()),
                        corpo: Cursor::new(Vec::<u8>::new()),
                    },
                    localizacao: Some(localizacao.to_owned()),
                })
            },
            |rede| {
                rede.chamadas.push("GET arvore");
                Ok(foto(1))
            },
        );
        assert!(matches!(numero, Ok(ref numero) if numero == "0000002"));
        assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
        assert_eq!(rede.submissoes.len(), 1);
    }
}

#[test]
fn html_200_sem_aceite_nao_comprova_post_ante_concorrencia() {
    // Resposta HTML genérica pode ser o cadastro reapresentado; a alteração
    // posterior da árvore pode ter sido causada por outra sessão compartilhada.
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
                    corpo: Cursor::new(b"<html><form>cadastro</form></html>".to_vec()),
                },
                localizacao: None,
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
    assert_eq!(rede.submissoes.len(), 1, "nunca repetir POST");
}

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
