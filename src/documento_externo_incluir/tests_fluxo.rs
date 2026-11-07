//! BDD da prévia Web externa (#332), transporte roteirizado, nenhuma rede real.
//! Oráculo Todos `tests/test_documento_juntar_010.py` (`UnB` SEI 4).
use super::*;
use crate::processo_concluir::tests_apoio as a;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_escolha() -> String {
    format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64)
    )
}
fn link_cadastro() -> String {
    format!(
        "controlador.php?acao=documento_receber&id_procedimento=10&infra_hash={}",
        "f".repeat(64)
    )
}
fn url(relativo: &str) -> String {
    format!("{}/{}", a::ORIGEM, relativo)
}
fn arvore() -> String {
    a::arvore(&[a::UNIDADE], &[acao_da_barra(&link_escolha())])
}
fn lista() -> String {
    let href = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><table id="tblSeries"><tr><td><a href="{href}">Externo</a></td></tr></table></html>"#
    )
}
fn cadastro(flag: &str, escolhido: bool) -> String {
    let action = link_cadastro().replace('&', "&amp;");
    let selecionado = if escolhido {
        "selected=\"selected\""
    } else {
        ""
    };
    format!(
        r#"<html><body>
    <form id="frmDocumentoCadastro" method="post" action="{action}">
      <select id="selSerie" name="selSerie">
        <option value="null"> </option><option value="293" {selecionado}>Anexo</option>
      </select>
      <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="{id}">
      <input type="hidden" id="hdnFlagDocumentoCadastro" name="hdnFlagDocumentoCadastro" value="{flag}">
      <input type="hidden" id="hdnAnexos" name="hdnAnexos" value="">
    </form><form id="frmAnexos"><input id="filArquivo" name="filArquivo" type="file"></form>
    <script>
    var arrExt = [];
    arrExt[0] = {{nome : "pdf", tamanho : 200}};
    objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);
    objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=FICTICIO');
    </script></body></html>"#,
        id = if escolhido { "293" } else { "" }
    )
}
fn executar(flag: &str) -> (Result<PreviaExterna, ErroPreviaExterna>, Vec<a::Pedido>) {
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore());
    respostas.push(a::ok(lista()));
    respostas.push(a::ok(cadastro(flag, false)));
    if flag == "1" {
        respostas.push(a::ok(cadastro("1", true)));
    }
    a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pdf = PdfLocal {
            nome: "prova.pdf".into(),
            tamanho_bytes: 68,
            sha256: "ficticio".into(),
        };
        executar_previa(sessao, nup, None, "Anexo", &pdf, indice, rede)
    })
}
#[test]
fn dado_sei4_quando_consulto_previa_entao_recarrega_tipo_sem_upload_nem_cadastro() {
    let (resultado, pedidos) = executar("1");
    let previa = resultado.unwrap_or_else(|e| panic!("prévia recusada: {e:?}"));
    assert_eq!(previa.id_tipo, "293");
    assert_eq!(previa.limite_pdf_bytes, 200 * 1024 * 1024);
    assert_eq!(previa.unidade, a::UNIDADE);
    let mut esperados = a::pedidos_da_abertura();
    esperados.push(a::get(&url(&link_escolha())));
    esperados.push(a::get(&url(&link_cadastro())));
    esperados.push(a::post(
        &url(&link_cadastro()),
        "selSerie=293&hdnIdSerie=293&hdnFlagDocumentoCadastro=1&hdnAnexos=",
    ));
    assert_eq!(
        pedidos, esperados,
        "um POST de recarga, zero upload e cadastro definitivo"
    );
}
#[test]
fn dado_flag_de_gravacao_quando_consulto_previa_entao_nao_envio_post() {
    let (resultado, pedidos) = executar("2");
    assert!(matches!(
        resultado,
        Err(ErroPreviaExterna::TipoNaoOferecido)
    ));
    let mut esperados = a::pedidos_da_abertura();
    esperados.push(a::get(&url(&link_escolha())));
    esperados.push(a::get(&url(&link_cadastro())));
    assert_eq!(pedidos, esperados, "flag 2 bloqueada antes de POST");
}
