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
