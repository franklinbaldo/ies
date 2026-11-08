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
    registrar_oferta(
        &banco,
        &origem(),
        "ensaio",
        "42",
        Oferta {
            editavel: true,
            assinaturas,
        },
    )
    .unwrap();
    banco
}
fn alvo() -> &'static str {
    "/sei/controlador.php?acao=documento_editar&id_documento=42"
}
fn form(confirmar_assinaturas: bool) -> String {
    format!(
        "versao_base=2&html_base=%3Cp%3Eantes%3C%2Fp%3E&html_editado=%3Cp%3Edepois%3C%2Fp%3E&confirmar_assinaturas={}",
        u8::from(confirmar_assinaturas)
    )
}
#[test]
fn versao_nova_exige_releitura_do_conteudo() {
    let banco = preparar(0);
    let previa = responder("GET", alvo(), "", &banco, &origem(), "ensaio");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("&lt;p&gt;antes&lt;/p&gt;"));
    assert!(previa.corpo.contains("name='html_editado'"));
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42")
            .unwrap()
            .unwrap()
            .revisoes
            .len(),
        1
    );
    let salva = responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio");
    assert_eq!(salva.estado, Estado::Redirecionamento);
    assert!(salva.localizacao.unwrap().contains("versao=3"));
    let publicada = anti_ies::documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=3",
        &banco,
        &origem(),
        "ensaio",
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
    assert_eq!(
        responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado,
        Estado::Redirecionamento
    );
    assert_eq!(
        responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
    let errado = "versao_base=3&html_base=%3Cp%3Eterceiro%3C%2Fp%3E&html_editado=%3Cp%3Emeu%3C%2Fp%3E&confirmar_assinaturas=0";
    assert_eq!(
        responder("POST", alvo(), errado, &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
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
    assert!(
        previa
            .corpo
            .contains("<output id='assinaturasAtuais'>2</output>")
    );
    assert_eq!(
        responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42")
            .unwrap()
            .unwrap()
            .revisoes
            .len(),
        1
    );
    assert_eq!(
        responder("POST", alvo(), &form(true), &banco, &origem(), "ensaio").estado,
        Estado::Redirecionamento
    );
    assert_eq!(
        consultar_oferta(&banco, &origem(), "ensaio", "42")
            .unwrap()
            .unwrap()
            .assinaturas,
        0
    );
    let depois = responder("GET", alvo(), "", &banco, &origem(), "ensaio");
    assert_eq!(depois.estado, Estado::Ok);
    assert!(
        depois
            .corpo
            .contains("<output id='assinaturasAtuais'>0</output>")
    );
}
#[test]
fn oferta_nao_observada_nao_inventa_permissao() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();
    assert_eq!(
        responder("GET", alvo(), "", &banco, &origem(), "ensaio").estado,
        Estado::DadosInsuficientes
    );
    registrar_oferta(
        &banco,
        &origem(),
        "ensaio",
        "42",
        Oferta {
            editavel: false,
            assinaturas: 0,
        },
    )
    .unwrap();
    assert_eq!(
        responder("GET", alvo(), "", &banco, &origem(), "ensaio").estado,
        Estado::MetodoNaoPermitido
    );
    assert_eq!(
        responder("POST", alvo(), &form(false), &banco, &origem(), "ensaio").estado,
        Estado::MetodoNaoPermitido
    );
}

/// BDD-89-02: HTTP local persiste e relê uma edição maior que o limite
/// anterior de 8 KiB, no mesmo SQLite (não é prova de SEI real).
#[test]
fn bdd_89_02_editor_http_grava_html_maior_que_oito_kib_e_rele() {
    use anti_ies::atender_http;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    fn pedir(endereco: std::net::SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
        let mut fluxo = TcpStream::connect(endereco).unwrap();
        write!(
            fluxo,
            "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
            corpo.len()
        )
        .unwrap();
        fluxo.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        fluxo.read_to_string(&mut resposta).unwrap();
        resposta
    }

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let banco = preparar(0);
    let servidor = std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem(), "ensaio"))).unwrap();
        }
        ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap()
    });
    let texto = "texto".repeat(2400);
    let edicao = format!("<p>{texto}</p>");
    let corpo = format!(
        "versao_base=2&html_base=%3Cp%3Eantes%3C%2Fp%3E&html_editado=%3Cp%3E{texto}%3C%2Fp%3E"
    );
    assert!(corpo.len() > 8192);
    let formulario = pedir(endereco, "GET", alvo(), "");
    assert!(formulario.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(formulario.contains("frmDocumentoEditar"));
    let publicada = pedir(endereco, "POST", alvo(), &corpo);
    assert!(
        publicada.starts_with("HTTP/1.1 302 Found\r\n"),
        "{publicada}"
    );
    let lida = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=3",
        "",
    );
    assert!(lida.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(lida.contains(&edicao));
    let documento = servidor.join().unwrap();
    assert_eq!(documento.revisoes.len(), 2);
    assert_eq!(documento.revisoes[0].html_corpo, "<p>antes</p>");
    assert_eq!(documento.revisoes[1].html_corpo, edicao);
}

/// Conteúdo literal pode mencionar o nome de um parâmetro sem ser credencial.
#[test]
fn corpo_legitimo_com_nome_de_parametro_e_editavel() {
    let banco = preparar(0);
    let form = "versao_base=2&html_base=%3Cp%3Eantes%3C%2Fp%3E&html_editado=%3Cp%3Einfra_hash%3C%2Fp%3E&confirmar_assinaturas=0";
    let resposta = responder("POST", alvo(), form, &banco, &origem(), "ensaio");
    assert_eq!(resposta.estado, Estado::Redirecionamento);
    let documento = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(documento.revisoes.len(), 2);
    assert_eq!(documento.revisoes[1].html_corpo, "<p>infra_hash</p>");
}

#[test]
fn editor_aceita_limpar_o_corpo_e_preserva_a_versao_anterior() {
    let banco = preparar(0);
    let formulario =
        "versao_base=2&html_base=%3Cp%3Eantes%3C%2Fp%3E&html_editado=&confirmar_assinaturas=0";
    let resposta = responder("POST", alvo(), formulario, &banco, &origem(), "ensaio");
    assert_eq!(resposta.estado, Estado::Redirecionamento);
    let documento = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(documento.revisoes.len(), 2);
    assert_eq!(documento.revisoes[0].html_corpo, "<p>antes</p>");
    assert_eq!(documento.revisoes[1].html_corpo, "");
    let versao = anti_ies::documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=3",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(versao.estado, Estado::Ok);
    assert!(versao.corpo.contains("<main id='corpoEditavel'></main>"));
}

#[test]
fn editor_preserva_secao_inalterada_em_documento_multissecao() {
    use iesei::editor_documento::{Edicao, publicar};

    let base =
        "<section id='cabecalho'><p>Fixo</p></section><section id='texto'><p>Antigo</p></section>";
    let editado =
        "<section id='cabecalho'><p>Fixo</p></section><section id='texto'><p>Novo</p></section>";
    let mut documento = documento();
    documento.revisoes[0].html_corpo = base.into();
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento).unwrap();
    registrar_oferta(
        &banco,
        &origem(),
        "ensaio",
        "42",
        Oferta {
            editavel: true,
            assinaturas: 0,
        },
    )
    .unwrap();

    let publicada = publicar(
        &banco,
        &origem(),
        "ensaio",
        "42",
        Edicao {
            versao_base: NonZeroU32::new(2).unwrap(),
            html_base: base,
            html_editado: editado,
            confirmar_assinaturas: false,
        },
    )
    .unwrap();
    assert_eq!(publicada.versao_nova.get(), 3);
    let revisoes = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(revisoes.revisoes[0].html_corpo, base);
    assert_eq!(revisoes.revisoes[1].html_corpo, editado);
    let pagina = anti_ies::documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=3",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(pagina.estado, Estado::Ok);
    assert!(pagina.corpo.contains(editado));
}

#[test]
fn editor_4_emula_formulario_observado_e_publica_versao_por_estado() {
    // Fonte estrutural: docs/como-o-sei-funciona/editor-do-documento.md
    // (Todos tests/paginas_reais/editor_montar.html, UnB 4.0.9).
    // Não é replay do membro JSON da cassette.
    let banco = preparar(0);
    let montagem = responder(
        "GET",
        "/sei/controlador.php?acao=editor_montar&id_documento=42",
        "",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(montagem.estado, Estado::Ok);
    assert!(montagem.corpo.contains("id='frmEditor'"));
    assert!(montagem.corpo.contains("target='ifrEditorSalvar'"));
    assert!(montagem.corpo.contains("acao=editor_salvar"));
    assert!(montagem.corpo.contains("name='txaEditor_1'"));
    assert!(montagem.corpo.contains("name='hdnVersao' value='2'"));
    assert!(
        montagem
            .corpo
            .contains("name='hdnIgnorarNovaVersao' value='N'")
    );
    let rota = "/sei/editor/editor_processar.php?acao=editor_salvar&id_documento=42";
    let edicao = "hdnVersao=2&hdnIgnorarNovaVersao=N&txaEditor_1=%3Cp%3Edepois%3C%2Fp%3E";
    let publicada = responder("POST", rota, edicao, &banco, &origem(), "ensaio");
    assert_eq!(publicada.estado, Estado::Ok);
    assert_eq!(publicada.corpo, "OK 3");
    let documento = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(documento.revisoes.len(), 2);
    assert_eq!(documento.revisoes[0].html_corpo, "<p>antes</p>");
    assert_eq!(documento.revisoes[1].html_corpo, "<p>depois</p>");
    // Repetir POST de versão ultrapassada não volta a publicar.
    assert_eq!(
        responder("POST", rota, edicao, &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
    assert_eq!(
        responder(
            "POST",
            rota,
            "hdnVersao=3&hdnIgnorarNovaVersao=S&txaEditor_1=alterado",
            &banco,
            &origem(),
            "ensaio"
        )
        .estado,
        Estado::RequisicaoInvalida
    );
}

#[test]
fn editor_4_documento_assinado_exige_confirmacao_no_post() {
    let banco = preparar(2);
    let rota = "/sei/editor/editor_processar.php?acao=editor_salvar&id_documento=42";
    let corpo = "hdnVersao=2&hdnIgnorarNovaVersao=N&txaEditor_1=%3Cp%3Edepois%3C%2Fp%3E";
    assert_eq!(
        responder("POST", rota, corpo, &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
    let inicial = ler(&banco, &origem(), "ensaio", "42").unwrap().unwrap();
    assert_eq!(inicial.revisoes.len(), 1);
    let autorizado = format!("{corpo}&confirmar_assinaturas=1");
    let salvo = responder("POST", rota, &autorizado, &banco, &origem(), "ensaio");
    assert_eq!(salvo.estado, Estado::Ok);
    assert_eq!(salvo.corpo, "OK 3");
    assert_eq!(
        consultar_oferta(&banco, &origem(), "ensaio", "42")
            .unwrap()
            .unwrap()
            .assinaturas,
        0
    );
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42")
            .unwrap()
            .unwrap()
            .revisoes
            .len(),
        2
    );
}
