//! Ensaios sintéticos da fronteira real de escrita e verificação (#246).
//! Casos de Todos `src/todos/documentos.py::_conferir_criacao` @323eb01.
use super::*;
use crate::arvore_processo::{DocumentoNaArvore, NaturezaDoDocumento};
use crate::formulario_web::{CodificacaoDaTela, RespostaDoPost};
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb};
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
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: 200,
            mime: "text/html; charset=UTF-8".to_owned(),
            content_length: None,
            localizacao: None,
            corpo: Cursor::new(b"<html><body>documento salvo</body></html>".to_vec()),
        },
        localizacao: None,
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
    assert!(matches!(resultado, Err(ErroPreviaCriacao::EfeitoIncerto)));
    assert_eq!(rede.chamadas, ["POST criador", "GET arvore"]);
    assert_eq!(rede.submissoes.len(), 1);
}
#[test]
fn dois_documentos_novos_sao_efeito_incerto_apos_post_unico() {
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
    assert!(matches!(resultado, Err(ErroPreviaCriacao::EfeitoIncerto)));
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
            Err(())
        },
    );
    assert!(matches!(resultado, Err(ErroPreviaCriacao::EfeitoIncerto)));
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
        Err(ErroPreviaCriacao::ArvoreIncompleta)
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
        Err(ErroPreviaCriacao::FotoInicialInconsistente)
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
        Err(ErroPreviaCriacao::FotoInicialInconsistente)
    ));
    assert!(
        rede.chamadas.is_empty(),
        "identidade ausente não autoriza POST"
    );
}
