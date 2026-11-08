//! BDD-88-01/03: resposta do anti-IES atravessa os leitores reais do IES.
use super::{VersaoDocumento, interpretar_historico};
use crate::documento::NumeroDocumento;
use crate::documento_web::{identidade_do_titulo, texto_do_corpo};
use crate::rodape_do_documento::versao_do_rodape;
use anti_ies::Estado;
use anti_ies::documento_versoes::responder;
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

#[test]
fn bdd_88_01_arvore_anti_ies_lida_pelo_ies() {
    use crate::arvore_processo::{
        ConteudoNaArvore, ler_arvore, ler_barra_do_documento, ler_conteudo_na_arvore,
    };
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let documento = DocumentoSimulado {
        instalacao: origem.clone(),
        id_documento: "42".into(),
        id_procedimento: "7".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Ofício".into(),
        revisoes: vec![Revisao {
            versao: NonZeroU32::new(3).unwrap(),
            usuario: "Bia".into(),
            unidade: "PPI".into(),
            modificacao: "03/11/2026".into(),
            html_corpo: "<p>Terceira redação</p>".into(),
        }],
    };
    guardar(&mut banco, "bdd-88", &documento).unwrap();
    let resposta = anti_ies::documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=42",
        &banco,
        &origem,
        "bdd-88",
    );
    assert_eq!(resposta.estado, Estado::Ok);
    let a = ler_arvore(&resposta.corpo, &documento.numero_processo, "7").unwrap();
    assert_eq!(a.documentos.len(), 1);
    assert_eq!(a.documentos[0].numero_sei.as_deref(), Some("0011519"));
    assert!(
        ler_barra_do_documento(&origem, &resposta.corpo, "42", "7")
            .unwrap()
            .versoes
            .is_some()
    );
    assert!(matches!(
        ler_conteudo_na_arvore(&origem, &resposta.corpo, "42").unwrap(),
        ConteudoNaArvore::Visualizar(_)
    ));
}

#[test]
fn bdd_88_01_leitor_ies_confere_lista_e_visualiza_versao_servida() {
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(
        &mut banco,
        "bdd-88",
        &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "42".into(),
            id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011519".into(),
            tipo: "Ofício".into(),
            revisoes: vec![
                Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Ana".into(),
                    unidade: "PPI".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: "<p>Primeira redação</p>".into(),
                },
                Revisao {
                    versao: NonZeroU32::new(3).unwrap(),
                    usuario: "Ana".into(),
                    unidade: "PPI".into(),
                    modificacao: "03/11/2026".into(),
                    html_corpo: "<p>Terceira redação</p>".into(),
                },
            ],
        },
    )
    .unwrap();

    let lista = responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        &banco,
        &origem,
        "bdd-88",
    );
    assert_eq!(lista.estado, Estado::Ok);
    let numero = NumeroDocumento::novo("0011519").unwrap();
    let historico = interpretar_historico(&lista.corpo, &numero, "42", None).unwrap();
    assert_eq!(historico.atual().unwrap().numero(), 3);
    let antiga = historico
        .selecionar(VersaoDocumento::nova(1).unwrap())
        .unwrap();
    let link = antiga.link_privado();
    assert!(link.contains("id_procedimento=7"));
    assert!(
        !link.contains("&amp;"),
        "o enlace entregue ao leitor é URL, não entidade HTML"
    );

    let resposta = responder("GET", &format!("/sei/{link}"), &banco, &origem, "bdd-88");
    assert_eq!(resposta.estado, Estado::Ok);
    let identidade = identidade_do_titulo(&resposta.corpo).unwrap();
    assert_eq!(identidade.numero, "0011519");
    assert_eq!(identidade.tipo, "Ofício");
    assert_eq!(
        versao_do_rodape(&resposta.corpo).map(VersaoDocumento::numero),
        Some(1)
    );
    let texto = texto_do_corpo(&resposta.corpo).unwrap();
    assert!(texto.contains("Primeira redação"));
    assert!(!texto.contains("Terceira redação"));
}
