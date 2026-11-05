use super::*;

fn lista_41() -> String {
    // Caso: Todos tests/test_comentarios_010.py @323eb01 (UFRJ SEI 4.1).
    r#"<html><body>
      <form id="frmComentarioLista">
        <input name="hdnVisualizacao" type="hidden" value="P">
        <table id="tblComentarios">
          <caption>Lista de Comentários (1 registro):</caption>
          <thead><tr><th></th><th>Data</th><th>Unidade</th><th>Usuário</th><th>Ações</th></tr></thead>
          <tbody>
            <tr><td><input type="checkbox" value="42"></td>
                <td>16/10/2026 09:14:44</td><td>ADC</td><td>pessoa</td><td></td></tr>
            <tr class="trComentario"><td colspan="5">Comentário fictício do teste automatizado do Todos.</td></tr>
          </tbody>
        </table>
      </form></body></html>"#.to_owned()
}

fn lista_50() -> String {
    // Caso: Todos src/todos/comentarios.py @323eb01 (GO SEI 5.0).
    r#"<form id='frmComentarioLista'>
        <input type="hidden" name="hdnVisualizacao" value="T">
        <table id="tblComentarios">
          <caption>Comentários (1 registro):</caption>
          <tr><th></th><th>Data</th><th>Unidade</th><th>Usuário</th><th>Protocolo</th><th>Tipo</th><th>Texto</th><th>Ações</th></tr>
          <tr name="trComentario42"><td><input type="checkbox" value="42"></td>
            <td>19/10/2026 10:30:00</td><td>GAB</td><td>u</td><td>000002</td><td>Despacho</td>
            <td>Texto de &amp; teste</td><td></td></tr>
        </table>
      </form>"#.to_owned()
}

fn lidos(html: &str) -> ComentariosLidos {
    match ler_tela(html).expect("HTML sintético reconhecido") {
        TelaDosComentarios::Lista(lidos) => lidos,
        TelaDosComentarios::CadastroSemComentarios => panic!("lista esperada"),
    }
}

#[test]
fn lista_de_4x_liga_linha_de_texto_ao_comentario() {
    let lidos = lidos(&lista_41());
    assert_eq!(lidos.total_declarado, 1);
    assert_eq!(lidos.visao, VisaoDosComentarios::Processo);
    assert_eq!(lidos.itens[0].id, "42");
    assert_eq!(lidos.itens[0].unidade, "ADC");
    assert_eq!(
        lidos.itens[0].texto,
        "Comentário fictício do teste automatizado do Todos."
    );
    assert_eq!(lidos.itens[0].protocolo, None);
}

#[test]
fn lista_de_5x_usa_coluna_texto_e_protocolo_sem_duplicar_linha() {
    let lidos = lidos(&lista_50());
    assert_eq!(lidos.total_declarado, 1);
    assert_eq!(lidos.visao, VisaoDosComentarios::Todos);
    assert_eq!(lidos.itens[0].texto, "Texto de & teste");
    assert_eq!(lidos.itens[0].protocolo.as_deref(), Some("000002"));
    assert_eq!(lidos.itens[0].tipo.as_deref(), Some("Despacho"));
}

#[test]
fn cadastro_de_lista_vazia_nao_se_confunde_com_tela_desconhecida() {
    // Caso: Todos tests/test_comentarios_010.py @323eb01.
    let html = r#"<form id="frmComentarioCadastro">
      <textarea name="txaDescricao"></textarea>
      <input name="sbmCadastrarComentario" type="submit" value="Salvar">
      </form>"#;
    assert_eq!(
        ler_tela(html),
        Ok(TelaDosComentarios::CadastroSemComentarios)
    );
    assert_eq!(
        ler_tela("<form id='frmComentarioCadastro'></form>"),
        Err(ErroComentarios::TelaNaoReconhecida)
    );
}

#[test]
fn tabela_ausente_nao_significa_lista_vazia() {
    let html = lista_41().replace("<table id=\"tblComentarios\">", "<table>");
    assert_eq!(ler_tela(&html), Err(ErroComentarios::TabelaAusente));
}

#[test]
fn total_maior_que_uma_pagina_e_lista_incompleta() {
    // Caso: Todos tests/test_comentarios_010.py @323eb01.
    let html = lista_41().replace("(1 registro)", "(2 registros)");
    assert_eq!(ler_tela(&html), Err(ErroComentarios::ListaIncompleta));
}

#[test]
fn dois_comentarios_com_mesmo_id_sao_recusados() {
    let linha = r#"<tr><td><input type="checkbox" value="42"></td>
            <td>16/10/2026 10:00:00</td><td>ADC</td><td>u</td><td></td></tr>
            <tr class="trComentario"><td colspan="5">Outro</td></tr>"#;
    let html = lista_41()
        .replace("(1 registro)", "(2 registros)")
        .replace("</tbody>", &format!("{linha}</tbody>"));
    assert_eq!(
        ler_tela(&html),
        Err(ErroComentarios::IdDoComentarioRepetido)
    );
}

#[test]
fn linha_de_4x_sem_texto_e_recusada() {
    let html = lista_41().replace("class=\"trComentario\"", "class=\"outra\"");
    assert_eq!(ler_tela(&html), Err(ErroComentarios::LinhaInvalida));
}

#[test]
fn visao_ausente_ou_repetida_e_recusada() {
    let html = lista_41().replace("value=\"P\"", "value=\"X\"");
    assert_eq!(ler_tela(&html), Err(ErroComentarios::VisaoNaoPublicada));
    let html = lista_41().replace(
        "<input name=\"hdnVisualizacao\"",
        "<input name=\"hdnVisualizacao\" value=\"T\"><input name=\"hdnVisualizacao\"",
    );
    assert_eq!(ler_tela(&html), Err(ErroComentarios::VisaoNaoPublicada));
}

#[test]
fn id_do_checkbox_deve_ser_decimal_e_inequivoco() {
    assert_eq!(
        ler_tela(&lista_41().replace("value=\"42\"", "value=\"42?\"")),
        Err(ErroComentarios::LinhaInvalida)
    );
    assert_eq!(
        ler_tela(&lista_41().replace(
            "<input type=\"checkbox\" value=\"42\">",
            "<input type=\"checkbox\" value=\"42\" value=\"43\">"
        )),
        Err(ErroComentarios::HtmlNaoInterpretado)
    );
}
