//! BDD derivado de Todos `tests/test_documento_juntar_010.py` (`UnB` SEI 4)
//! e `tests/test_documento_juntar_sei5_010.py` (PEN3 SEI 5).
//! Fixtures sintéticas: nenhum cookie, URL assinada verdadeira ou rede.
use super::*;

fn cadastro(terceiro: &str, selecionado: &str, extensoes: &str) -> String {
    format!(
        r#"<html><body>
<form id="frmDocumentoCadastro" method="post" action="controlador.php?acao=documento_receber&id_procedimento=50961">
  <select id="selSerie" name="selSerie">
    <option value="null"> </option>
    <option value="293" {selecionado}>Anexo</option>
    <option value="291">Despacho</option>
  </select>
  <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="293" />
  <input type="hidden" id="hdnFlagDocumentoCadastro" name="hdnFlagDocumentoCadastro" value="1" />
  <input type="hidden" id="hdnAnexos" name="hdnAnexos" value="" />
</form><form id="frmAnexos"><input type="file" name="filArquivo" id="filArquivo" /></form>
<script type="text/javascript">
  var arrExt = [];
  {extensoes}
  objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);
  objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=FICTICIO'{terceiro});
</script></body></html>"#
    )
}

const PDF: &str = r#"arrExt[0] = {nome : "pdf", tamanho : 200};"#;

#[test]
fn dado_cadastro_unb_recarregado_quando_inspeciono_entao_encontro_tipo_limite_e_upload() {
    let result =
        observar_cadastro(&cadastro("", "selected=\"selected\"", PDF), "Anexo", 68).unwrap();
    assert_eq!(result.id_tipo, "293");
    assert_eq!(result.limite_pdf_bytes, 200 * 1024 * 1024);
    assert!(
        result
            .plano
            .url_upload()
            .contains("acao=documento_upload_anexo")
    );
}

#[test]
fn dado_cadastro_sei5_com_terceiro_argumento_quando_leio_entao_mantem_o_plano() {
    let result = observar_cadastro(
        &cadastro(", true", "selected=\"selected\"", PDF),
        "Anexo",
        68,
    )
    .unwrap();
    assert_eq!(result.id_tipo, "293");
}

#[test]
fn dado_tipo_nao_escolhido_quando_preparo_upload_entao_exijo_recarga_do_cadastro() {
    assert!(matches!(
        observar_cadastro(&cadastro("", "", PDF), "Anexo", 68),
        Err(FalhaCadastroExterno::TipoNaoRecarregado)
    ));
}

#[test]
fn dado_pdf_maior_que_limite_observado_quando_inspeciono_entao_recuso_antes_do_upload() {
    assert!(matches!(
        observar_cadastro(
            &cadastro("", "selected=\"selected\"", PDF),
            "Anexo",
            200 * 1024 * 1024 + 1
        ),
        Err(FalhaCadastroExterno::PdfAcimaDoLimite)
    ));
}

#[test]
fn dado_tipo_ausente_ou_duplicado_quando_inspeciono_entao_nao_invento_id() {
    let normal = cadastro("", "selected=\"selected\"", PDF);
    assert!(matches!(
        observar_cadastro(&normal, "Parecer", 68),
        Err(FalhaCadastroExterno::TipoNaoOferecido)
    ));
    let duplicado = normal.replace(
        "<option value=\"291\">Despacho</option>",
        "<option value=\"291\">Anexo</option>",
    );
    assert!(matches!(
        observar_cadastro(&duplicado, "Anexo", 68),
        Err(FalhaCadastroExterno::TipoAmbiguo)
    ));
}

#[test]
fn dado_cadastro_sem_pdf_quando_avalio_entao_recuso_pdf_nao_oferecido() {
    let outro = r#"arrExt[0] = {nome : "txt", tamanho : 200};"#;
    assert!(matches!(
        observar_cadastro(&cadastro("", "selected=\"selected\"", outro), "Anexo", 68),
        Err(FalhaCadastroExterno::PdfNaoOferecido)
    ));
}

#[test]
fn dado_tipo_desabilitado_quando_consulto_previa_entao_nao_e_oferecido() {
    // BDD-332-02: um tipo desabilitado nao e opcao oferecida ao usuario.
    let html = cadastro("", "selected=\"selected\" disabled", PDF);
    assert!(matches!(
        observar_cadastro(&html, "Anexo", 68),
        Err(FalhaCadastroExterno::TipoNaoOferecido)
    ));
}

#[test]
fn dado_formulario_com_campos_quando_previa_entao_exibe_so_controles_ativos_sem_valores() {
    // BDD-332-02: saída da prévia publica os campos ofertados pelo cadastro.
    let pagina = cadastro("", "selected=\"selected\"", PDF).replacen(
        "</form>",
        r#"<input type="text" name="txtDataElaboracao" value="confidencial">
<select name="selTipoConferencia"><option value="0">Original</option></select>
<input type="radio" name="rdoFormato" value="N" checked>
<input type="radio" name="rdoFormato" value="D">
<input type="radio" name="rdoNivelAcesso" value="0" checked>
<input type="hidden" name="hdnExtra" value="nao-publicar">
<input type="text" name="desabilitado" disabled>
</form>"#,
        1,
    );
    let cadastro = observar_cadastro(&pagina, "Anexo", 68).unwrap();
    assert_eq!(
        cadastro.campos_oferecidos,
        [
            "selSerie",
            "txtDataElaboracao",
            "selTipoConferencia",
            "rdoFormato",
            "rdoNivelAcesso"
        ]
    );
    assert!(
        !cadastro
            .campos_oferecidos
            .iter()
            .any(|campo| campo.contains("confidencial"))
    );
}

#[test]
fn bdd_332_02_leitor_ies_consume_cadastro_gerado_do_sqlite_do_anti_ies() {
    use crate::documento_externo_incluir::escolha::{
        EscolhaExterna, escolher_externo, identificar_tipo,
    };
    use iesei::banco_cassettes::iniciar;
    use iesei::instalacao::Instalacao;
    use iesei::pdf_externo::{Processo, TipoExterno, semear};
    use rusqlite::Connection;

    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    semear(
        &banco,
        &origem,
        "ensaio",
        &Processo {
            id: "7".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 200 * 1024 * 1024,
        },
    )
    .unwrap();
    let renderizar = |metodo, alvo, corpo: &[u8]| {
        anti_ies::pdf_externo::atender(metodo, alvo, corpo, &banco, &origem, "ensaio")
            .expect("rota disponível")
            .corpo
    };
    let escolha = renderizar(
        "GET",
        "/sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=7",
        b"",
    );
    assert!(matches!(
        escolher_externo(&escolha),
        Ok(EscolhaExterna::Url(_))
    ));
    let formulario = renderizar(
        "GET",
        "/sei/controlador.php?acao=documento_receber&id_procedimento=7",
        b"",
    );
    assert_eq!(identificar_tipo(&formulario, "Anexo"), Ok("293".into()));
    let recarregado = renderizar(
        "POST",
        "/sei/controlador.php?acao=documento_receber&id_procedimento=7",
        b"hdnFlagDocumentoCadastro=1&selSerie=293",
    );
    let previa = observar_cadastro(&recarregado, "Anexo", 68).unwrap();
    assert_eq!(previa.id_tipo, "293");
    assert_eq!(previa.limite_pdf_bytes, 200 * 1024 * 1024);
    assert!(
        previa
            .campos_oferecidos
            .contains(&"txtDataElaboracao".to_owned())
    );
    assert!(
        previa
            .campos_oferecidos
            .contains(&"rdoNivelAcesso".to_owned())
    );
}

#[test]
fn bdd_332_03_ies_interpreta_upload_e_cadastra_no_anti_ies_com_redirect_conferivel() {
    // Integra componentes reais do consumidor IES e do provedor anti-IES
    // com estado SQLite compartilhado, sem rede externa nem credenciais.
    use crate::documento_externo_incluir::{
        cadastro_envio::preparar_cadastro_definitivo, recibo::interpretar_resposta,
    };
    use crate::formulario_web::{CodificacaoDaTela, TelaServida};
    use iesei::banco_cassettes::iniciar;
    use iesei::instalacao::Instalacao;
    use iesei::pdf_externo::{self as modelo, Processo, TipoExterno};
    use rusqlite::Connection;

    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    modelo::semear(
        &banco,
        &origem,
        "ensaio",
        &Processo {
            id: "7".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 200 * 1024 * 1024,
        },
    )
    .unwrap();
    let alvo = "/sei/controlador.php?acao=documento_receber&id_procedimento=7";
    let recarregado = anti_ies::pdf_externo::atender(
        "POST",
        alvo,
        b"hdnFlagDocumentoCadastro=1&selSerie=293",
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert_eq!(recarregado.estado, anti_ies::Estado::Ok);

    let pdf = b"%PDF-1.7\nconteudo";
    let previa = observar_cadastro(&recarregado.corpo, "Anexo", pdf.len() as u64).unwrap();
    let multipart = b"--B\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"prova.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.7\nconteudo\r\n--B--\r\n";
    let upload = anti_ies::pdf_externo::atender(
        "POST",
        "/sei/controlador.php?acao=documento_upload_anexo&id_procedimento=7",
        multipart,
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    let recibo = interpretar_resposta(200, &upload.corpo, "prova.pdf", pdf.len() as u64).unwrap();
    let antes = modelo::documentos(&banco, &origem, "ensaio", "7").unwrap();
    assert_eq!(antes, []);

    let url_cadastro =
        "https://sei.exemplo/sei/controlador.php?acao=documento_receber&id_procedimento=7";
    let tela = TelaServida {
        url: url_cadastro,
        html: &recarregado.corpo,
        codificacao: CodificacaoDaTela::Utf8,
    };
    let submissao =
        preparar_cadastro_definitivo(&origem, &tela, "TESTE", &previa, &recibo).unwrap();
    let definitivo =
        anti_ies::pdf_externo::atender("POST", alvo, submissao.corpo(), &banco, &origem, "ensaio")
            .unwrap();
    let documentos = modelo::documentos(&banco, &origem, "ensaio", "7").unwrap();
    assert_eq!(documentos.len(), 1);
    assert_eq!(documentos[0].tipo, "Anexo");
    assert_eq!(documentos[0].nome, "prova.pdf");
    // O IES não considera HTTP 200 genérico prova da autoria do cadastro:
    // exige redirect ao documento recém-observado na árvore.
    assert_eq!(definitivo.estado, anti_ies::Estado::Redirecionamento);
    assert!(
        crate::documento_criar::redirecionamento_do_mesmo_controlador(
            definitivo.localizacao.as_deref().unwrap(),
            submissao.url(),
            &documentos[0].id_publico(),
        )
    );
}

#[test]
fn bdd_332_03_ies_le_arvore_real_depois_do_cadastro_simulado() {
    use crate::arvore_processo::ler_arvore;
    use iesei::banco_cassettes::iniciar;
    use iesei::documentos_simulados::{self as interno, Tipo};
    use iesei::instalacao::Instalacao;
    use iesei::pdf_externo::{self as externo, Processo, TipoExterno};
    use rusqlite::Connection;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    externo::semear(
        &banco,
        &origem,
        "caso",
        &Processo {
            id: "7".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 100_000,
        },
    )
    .unwrap();
    interno::registrar_tipo(
        &banco,
        &origem,
        "caso",
        "7",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    interno::criar(&banco, &origem, "caso", "7", "291", "Primeiro").unwrap();
    let temporario = externo::receber_conteudo(
        &banco,
        &origem,
        "caso",
        "7",
        "prova.pdf",
        b"%PDF-1.7\nconteudo",
    )
    .unwrap();
    externo::cadastrar(&banco, &origem, "caso", "7", "293", temporario.id).unwrap();

    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let (mut fluxo, _) = servidor.accept().unwrap();
        anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, "caso"))).unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    cliente.write_all(
        b"GET /sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento= HTTP/1.1\r\nHost: localhost\r\n\r\n",
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    tarefa.join().unwrap();
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    let (_, html) = resposta.split_once("\r\n\r\n").unwrap();
    let arvore = ler_arvore(html, "00001.000001/2026-01", "7").unwrap();
    assert_eq!(arvore.documentos.len(), 2);
    assert!(
        arvore
            .documentos
            .iter()
            .any(|d| d.rotulo == "Despacho 1000001")
    );
    assert!(
        arvore
            .documentos
            .iter()
            .any(|d| d.rotulo == "Anexo (050000001)")
    );
    // A confirmação da CLI exige natureza PDF, não apenas rótulo/número.
    // O leitor IES deve reconhecer o ícone publicado pelo anti-IES.
    let pdf = arvore
        .documentos
        .iter()
        .find(|d| d.rotulo == "Anexo (050000001)")
        .expect("documento definitivo lido");
    assert_eq!(pdf.natureza.icone.as_deref(), Some("documento_pdf"));
    assert_eq!(pdf.natureza.cancelado, Some(false));
}

#[test]
fn dado_recarga_unb_com_opcao_nbsp_quando_inspeciono_entao_encontro_o_tipo() {
    // Recarga observada na UnB (cassette documento-mover-cancelar-unb,
    // interação 17): a opção vazia vem como `&nbsp;`.
    let html = cadastro("", "selected=\"selected\"", PDF).replace(
        r#"<option value="null"> </option>"#,
        r#"<option value="null">&nbsp;</option>"#,
    );
    let result = observar_cadastro(&html, "Anexo", 68).unwrap();
    assert_eq!(result.id_tipo, "293");
}
