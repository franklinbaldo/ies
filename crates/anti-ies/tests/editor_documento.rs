//! Testes técnicos do editor sintético para os BDD-89-02/03/04 da CLI.
use anti_ies::{Estado, editor_documento::responder};
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};
use iesei::editor_documento::{Oferta, consultar_oferta, registrar_oferta};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}
fn documento() -> DocumentoSimulado {
    DocumentoSimulado {
        instalacao: origem(),
        id_documento: "42".into(),
        id_procedimento: "7".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Despacho".into(),
        revisoes: vec![Revisao {
            versao: NonZeroU32::new(2).unwrap(),
            usuario: "Ana".into(),
            unidade: "TESTE".into(),
            modificacao: "01/11/2026".into(),
            html_corpo: "<p>antes</p>".into(),
        }],
    }
}
fn preparar(assinaturas: u32) -> Connection {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();
    registrar_oferta(&banco, &origem(), "ensaio", "42", Oferta { editavel: true, assinaturas }).unwrap();
    banco
}
fn alvo() -> &'static str {
    "/sei/controlador.php?acao=documento_editar&id_documento=42"
}
fn form(confirmar_assinaturas: bool) -> String {
    format!("versao_base=2&html_base=%3Cp%3Eantes%3C%2Fp%3E&html_editado=%3Cp%3Edepois%3C%2Fp%3E&confirmar_assinaturas={}", u8::from(confirmar_assinaturas))
}
#[test]
fn versao_nova_exige_releitura_do_conteudo() {
    let banco = preparar(0);
    let previa = responder("GET", alvo(), "", &banco, &origem(), "ensaio");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("&lt;p&gt;antes&lt;/p&gt;"));
    assert!(previa.corpo.contains("name='html_editado'"));
    assert_eq!(ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap().revisoes.len(), 1);
    let salva = responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio");
    assert_eq!(salva.estado, Estado::Redirecionamento);
    assert!(salva.localizacao.unwrap().contains("versao=3"));
    let publicada = anti_ies::documento_versoes::responder(
        "GET", "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=3",
        &banco, &origem(), "ensaio",
    );
    assert_eq!(publicada.estado, Estado::Ok);
    assert!(publicada.corpo.contains("<p>depois</p>"));
    let doc = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(doc.revisoes.len(), 2);
    assert_eq!(doc.revisoes[0].html_corpo, "<p>antes</p>");
}
#[test]
fn concorrencia_e_repeticao_nao_sobrescrevem() {
    let banco = preparar(0);
    assert_eq!(responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado, Estado::Redirecionamento);
    assert_eq!(responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado, Estado::RequisicaoInvalida);
    let errado = "versao_base=3&html_base=%3Cp%3Eterceiro%3C%2Fp%3E&html_editado=%3Cp%3Emeu%3C%2Fp%3E&confirmar_assinaturas=0";
    assert_eq!(responder("POST", alvo(), errado, &banco, &origem(), "ensaio").estado, Estado::RequisicaoInvalida);
    let doc = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(doc.revisoes.len(), 2);
    assert_eq!(doc.revisoes[1].html_corpo, "<p>depois</p>");
}
#[test]
fn editor_oferecido_aceita_assinado_com_consentimento() {
    let banco = preparar(2);
    let previa = responder("GET", alvo(), "", &banco, &origem(), "ensaio");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("2 assinatura"));
    assert_eq!(responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado, Estado::RequisicaoInvalida);
    assert_eq!(ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap().revisoes.len(), 1);
    assert_eq!(responder("POST", alvo(), &form(true), &banco, &origem(), "ensaio").estado, Estado::Redirecionamento);
    assert_eq!(consultar_oferta(&banco, &origem(), "ensaio", "42").unwrap().unwrap().assinaturas, 0);
}
#[test]
fn oferta_nao_observada_nao_inventa_permissao() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();
    assert_eq!(responder("GET", alvo(), "", &banco, &origem(), "ensaio").estado, Estado::DadosInsuficientes);
    registrar_oferta(&banco, &origem(), "ensaio", "42", Oferta { editavel: false, assinaturas: 0 }).unwrap();
    assert_eq!(responder("GET", alvo(), "", &banco, &origem(), "ensaio").estado, Estado::MetodoNaoPermitido);
    assert_eq!(responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado, Estado::MetodoNaoPermitido);
}
