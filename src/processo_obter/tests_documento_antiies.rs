//! #88: o leitor real do IES percorre as paginas sinteticas do mesmo SQLite.
//! Roteiro de transporte: prova do fluxo Rust, nao de HTTPS ou CLI externa.
use super::tests_apoio::*;
use super::{PedidoDeVersoes, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso};
use crate::arvore_processo::tests::NUP;
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use crate::processo_obter::obter_documento_com_versoes;
use crate::transporte::sessao::CookieDaSessao;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use rusqlite::Connection;
use std::num::NonZeroU32;

const DOC: &str = "0000101";

fn origem_e_documento() -> (Connection, iesei::instalacao::Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = instalacao();
    banco_cassettes::guardar(
        &mut banco,
        "bdd-88",
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova(UNIDADE).unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    documento_versoes::guardar(
        &mut banco,
        "bdd-88",
        &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "21".into(),
            id_procedimento: "10".into(),
            numero_processo: NUP.into(),
            numero: DOC.into(),
            tipo: "Despacho".into(),
            revisoes: vec![
                Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Ana".into(),
                    unidade: UNIDADE.into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: "<p>Primeira redação</p>".into(),
                },
                Revisao {
                    versao: NonZeroU32::new(3).unwrap(),
                    usuario: "Bia".into(),
                    unidade: UNIDADE.into(),
                    modificacao: "03/11/2026".into(),
                    html_corpo: "<p>Terceira redação</p>".into(),
                },
            ],
        },
    )
    .unwrap();
    (banco, origem)
}

/// Foto gerada pela leitura real: sem sobrescrita local e sem retry remoto.
fn conferir_foto_local(resultado: &super::DocumentoObtido) {
    let foto = resultado.versoes.foto.as_ref().unwrap();
    assert_eq!(foto.versao.numero(), 1);
    assert!(foto.html.contains("Primeira redação"));

    // BDD-88-02: a mesma resposta do anti-IES segue ate a foto da CLI.
    // A segunda leitura e idempotente; edicao local divergente permanece.
    let pasta = std::env::temp_dir().join(format!(
        "ies-anti-ies-88-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&pasta).unwrap();
    let lote = std::slice::from_ref(resultado);
    let primeira = crate::documento_versao_cli::gravar_fotos(&pasta, lote).unwrap();
    assert!(primeira[0].as_ref().unwrap().gravado);
    let destino = pasta.join("0000101.v1.html");
    assert!(
        std::fs::read_to_string(&destino)
            .unwrap()
            .contains("Primeira redação")
    );
    let repetida = crate::documento_versao_cli::gravar_fotos(&pasta, lote).unwrap();
    assert!(!repetida[0].as_ref().unwrap().gravado);
    std::fs::write(&destino, "<html><body>Edicao local</body></html>").unwrap();
    assert_eq!(
        crate::documento_versao_cli::gravar_fotos(&pasta, lote),
        Err(crate::copia_documento::ErroCopia::FotoDivergente)
    );
    assert_eq!(
        std::fs::read_to_string(&destino).unwrap(),
        "<html><body>Edicao local</body></html>"
    );
    std::fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn bdd_88_01_02_leitor_rust_real_percorre_a_pesquisa_do_anti_ies() {
    let (banco, origem) = origem_e_documento();
    let controle = anti_ies::controlar(
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        &banco,
        &origem,
        "bdd-88",
    );
    let pesquisa = anti_ies::pesquisa_documento::responder(
        "POST",
        "/sei/controlador.php?acao=protocolo_pesquisa_rapida",
        "txtPesquisaRapida=0000101",
        &banco,
        &origem,
        "bdd-88",
    );
    let tela = anti_ies::documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_protocolo=21",
        &banco,
        &origem,
        "bdd-88",
    );
    let arvore = anti_ies::documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21",
        &banco,
        &origem,
        "bdd-88",
    );
    let historico = anti_ies::documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=21",
        &banco,
        &origem,
        "bdd-88",
    );
    let texto = anti_ies::documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=21&versao=1",
        &banco,
        &origem,
        "bdd-88",
    );
    assert_eq!(pesquisa.estado, anti_ies::Estado::Redirecionamento);
    for resposta in [&controle, &tela, &arvore, &historico, &texto] {
        assert_eq!(resposta.estado, anti_ies::Estado::Ok);
    }
    let respostas = vec![
        ok(controle.corpo),
        r302(pesquisa.localizacao.as_deref().unwrap()),
        ok(tela.corpo),
        ok(arvore.corpo),
        ok(historico.corpo),
        ok(texto.corpo),
    ];
    let amb = ambiente();
    let url = url_controle();
    let cookie = CookieDaSessao::ler(&amb.cookie);
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &cookie,
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        processo: None,
        anexo: PedidoDoAnexo::default(),
    };
    let versoes = PedidoDeVersoes {
        versao: Some(VersaoDocumento::nova(1).unwrap()),
        foto: true,
        cache: false,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let resultado =
        obter_documento_com_versoes(&sessao, &pedido, versoes, &amb.indice, &mut roteiro)
            .expect("o IES precisa ler a versao 1 simulada, sem substituir pela atual");
    assert_eq!(resultado.numero, DOC);
    assert_eq!(resultado.requisicoes, 6);
    assert!(resultado.texto.contains("Primeira redação"));
    assert!(!resultado.texto.contains("Terceira redação"));
    assert_eq!(resultado.versoes.lida.unwrap().numero(), 1);
    assert_eq!(resultado.versoes.atual.unwrap().numero(), 3);
    conferir_foto_local(&resultado);
    assert_eq!(roteiro.respostas_restantes(), 0);
    assert_eq!(roteiro.pedidos.len(), 6);
}
