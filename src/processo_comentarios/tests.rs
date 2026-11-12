use super::*;
use crate::tela_html::HtmlServido;

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
    match ler_tela(&HtmlServido::de_teste(html)).expect("HTML sintético reconhecido") {
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
    let html = HtmlServido::de_teste(
        r#"<form id="frmComentarioCadastro">
      <textarea name="txaDescricao"></textarea>
      <input name="sbmCadastrarComentario" type="submit" value="Salvar">
      </form>"#,
    );
    assert_eq!(
        ler_tela(&html),
        Ok(TelaDosComentarios::CadastroSemComentarios)
    );
    assert_eq!(
        ler_tela(&HtmlServido::de_teste(
            "<form id='frmComentarioCadastro'></form>"
        )),
        Err(ErroComentarios::TelaDeComentariosNaoReconhecida)
    );
}

#[test]
fn tabela_ausente_nao_significa_lista_vazia() {
    let html =
        HtmlServido::de_teste(&lista_41().replace("<table id=\"tblComentarios\">", "<table>"));
    assert_eq!(
        ler_tela(&html),
        Err(ErroComentarios::TabelaDeComentariosAusente)
    );
}

#[test]
fn total_maior_que_uma_pagina_e_lista_incompleta() {
    // Caso: Todos tests/test_comentarios_010.py @323eb01.
    let html = HtmlServido::de_teste(&lista_41().replace("(1 registro)", "(2 registros)"));
    assert_eq!(ler_tela(&html), Err(ErroComentarios::ListaIncompleta));
}

#[test]
fn dois_comentarios_com_mesmo_id_sao_recusados() {
    let linha = r#"<tr><td><input type="checkbox" value="42"></td>
            <td>16/10/2026 10:00:00</td><td>ADC</td><td>u</td><td></td></tr>
            <tr class="trComentario"><td colspan="5">Outro</td></tr>"#;
    let html = HtmlServido::de_teste(
        &lista_41()
            .replace("(1 registro)", "(2 registros)")
            .replace("</tbody>", &format!("{linha}</tbody>")),
    );
    assert_eq!(
        ler_tela(&html),
        Err(ErroComentarios::IdDoComentarioRepetido)
    );
}

#[test]
fn linha_de_4x_sem_texto_e_recusada() {
    let html =
        HtmlServido::de_teste(&lista_41().replace("class=\"trComentario\"", "class=\"outra\""));
    assert_eq!(
        ler_tela(&html),
        Err(ErroComentarios::LinhaDoComentarioInvalida)
    );
}

#[test]
fn visao_ausente_ou_repetida_e_recusada() {
    let html = HtmlServido::de_teste(&lista_41().replace("value=\"P\"", "value=\"X\""));
    assert_eq!(ler_tela(&html), Err(ErroComentarios::VisaoNaoPublicada));
    let html = HtmlServido::de_teste(&lista_41().replace(
        "<input name=\"hdnVisualizacao\"",
        "<input name=\"hdnVisualizacao\" value=\"T\"><input name=\"hdnVisualizacao\"",
    ));
    assert_eq!(ler_tela(&html), Err(ErroComentarios::VisaoNaoPublicada));
}

#[test]
fn id_do_checkbox_deve_ser_decimal_e_inequivoco() {
    assert_eq!(
        ler_tela(&HtmlServido::de_teste(
            &lista_41().replace("value=\"42\"", "value=\"42?\"")
        )),
        Err(ErroComentarios::LinhaDoComentarioInvalida)
    );
    // Repetido na tag, vale o primeiro, como no navegador.
    assert_eq!(
        ler_tela(&HtmlServido::de_teste(&lista_41().replace(
            "<input type=\"checkbox\" value=\"42\">",
            "<input type=\"checkbox\" value=\"42\" value=\"43\">"
        ))),
        ler_tela(&HtmlServido::de_teste(&lista_41()))
    );
}

// A barra da árvore é a fonte da URL, não uma URL montada a partir do NUP.
// Caso Todos tests/test_comentarios_010.py @323eb01 (ação oferecida).
fn arvore_com_comentarios(acoes: &[String]) -> String {
    use crate::arvore_processo::tests::{NUP, arvore, no_processo};
    use crate::propriedades_processo::tests_apoio::{barra, com_barra};
    let base = arvore(&[no_processo(0, "10", NUP)]);
    com_barra(&base, 0, &barra(acoes))
}

fn link_comentarios(id: &str) -> String {
    use crate::propriedades_processo::tests_apoio::{acao_da_barra, link_das_propriedades};
    acao_da_barra(&link_das_propriedades("comentario_listar", id))
}

fn link_da_arvore(html: &str) -> Result<Option<EnderecoDosComentarios>, ErroLinkDosComentarios> {
    let instalacao = crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei")
        .expect("origem sintética");
    crate::arvore_processo::ler_link_dos_comentarios(&instalacao, html, "10")
}

#[test]
fn pega_o_link_de_comentarios_assinado_da_barra_do_processo() {
    let href = "controlador.php?acao=comentario_listar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento=10&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash=".to_owned()
        + &"3".repeat(64);
    let extraido = link_da_arvore(&arvore_com_comentarios(&[link_comentarios("10")]))
        .expect("link da barra")
        .expect("ação publicada");
    assert_eq!(
        extraido.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{href}")
    );
    assert_eq!(
        format!("{extraido:?}"),
        "EnderecoDosComentarios(<redigido>)"
    );
}

#[test]
fn comentario_em_outro_processo_ou_repetido_e_recusa() {
    assert_eq!(
        link_da_arvore(&arvore_com_comentarios(&[link_comentarios("11")])),
        Err(ErroLinkDosComentarios::EnderecoDeOutroProcesso)
    );
    assert_eq!(
        link_da_arvore(&arvore_com_comentarios(&[
            link_comentarios("10"),
            link_comentarios("10"),
        ])),
        Err(ErroLinkDosComentarios::LinksDosComentariosRepetidosNaBarra)
    );
}

#[test]
fn barra_sem_comentarios_nao_inventa_endereco() {
    assert_eq!(link_da_arvore(&arvore_com_comentarios(&[])), Ok(None));
}

#[test]
fn cadastro_com_envio_em_botao_e_reconhecido() {
    // Caso: SEI publica o envio como `<button name="sbmCadastrarComentario">` (32 gravações).
    let html = HtmlServido::de_teste(
        r#"<form id="frmComentarioCadastro">
      <textarea name="txaDescricao"></textarea>
      <button type="submit" accesskey="S" name="sbmCadastrarComentario" value="Salvar" class="infraButton">Salvar</button>
      </form>"#,
    );
    assert_eq!(
        ler_tela(&html),
        Ok(TelaDosComentarios::CadastroSemComentarios)
    );
}

#[test]
fn cadastro_com_botao_de_outro_nome_e_recusado() {
    let html = HtmlServido::de_teste(
        r#"<form id="frmComentarioCadastro">
      <textarea name="txaDescricao"></textarea>
      <button type="submit" name="outro">Salvar</button>
      </form>"#,
    );
    assert_eq!(
        ler_tela(&html),
        Err(ErroComentarios::TelaDeComentariosNaoReconhecida)
    );
}

#[test]
fn visao_de_protocolo_a_e_lida_como_variante_propria() {
    // Caso: SEI, `hdnVisualizacao="A"`: lista de um protocolo, sem os links
    // "Ver todos"/"Ver somente do processo" e com o Novo de protocolo.
    let lidos = lidos(&lista_41().replace("value=\"P\"", "value=\"A\""));
    assert_eq!(lidos.visao, VisaoDosComentarios::Protocolo);
    assert_eq!(lidos.itens[0].id, "42");
}

#[test]
fn separador_entre_comentarios_da_visao_t_nao_e_item() {
    // Caso: SEI, visão T com dois comentários: `<tr class="trSeparadorComentario">`
    // vazia (`&nbsp;`) separa um comentário do seguinte.
    let linha = r#"<tr class="trSeparadorComentario"><td colspan="5">&nbsp;</td></tr>
            <tr><td><input type="checkbox" value="43"></td>
            <td>16/10/2026 10:00:00</td><td>ADC</td><td>u</td><td></td></tr>
            <tr class="trComentario"><td colspan="5">Segundo</td></tr>"#;
    let html = lista_41()
        .replace("(1 registro)", "(2 registros)")
        .replace("</tbody>", &format!("{linha}</tbody>"));
    let lidos = lidos(&html);
    assert_eq!(lidos.itens.len(), 2);
    assert_eq!(lidos.itens[1].id, "43");
    assert_eq!(lidos.itens[1].texto, "Segundo");
}

#[test]
fn separador_com_texto_ou_classe_parecida_e_recusado() {
    let com_texto = r#"<tr class="trSeparadorComentario"><td colspan="5">Texto</td></tr>
            <tr class="trComentario"><td colspan="5">x</td></tr>"#;
    let classe_parecida =
        r#"<tr class="trSeparadorComentarioExtra"><td colspan="5">&nbsp;</td></tr>"#;
    for linha in [com_texto, classe_parecida] {
        let html =
            HtmlServido::de_teste(&lista_41().replace("</tbody>", &format!("{linha}</tbody>")));
        assert_eq!(
            ler_tela(&html),
            Err(ErroComentarios::LinhaDoComentarioInvalida),
            "linha recusada: {linha}"
        );
    }
}
