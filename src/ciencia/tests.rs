//! Casos sintéticos inspirados em Todos `test_ciclo_ciencia_010.py` e
//! `test_documento_ciente_010.py` @323eb01. Sem cookies nem URLs assinadas.
use super::{AlvoDaCiencia as Alvo, ErroCiencia as E, atividade_do_aceite, atividade_na_lista};

fn aceite(acao: &str, id: &str, escopo: &str, atividade: &str) -> String {
    format!(
        "<html><script>function inicializar(){{\n atualizarArvore('controlador.php?acao={acao}&id_procedimento={id}&{escopo}&id_atividade={atividade}#ID-{atividade}');\n}}</script></html>"
    )
}

fn processo() -> Alvo<'static> {
    Alvo::Processo {
        id_procedimento: "10",
    }
}

fn documento() -> Alvo<'static> {
    Alvo::Documento {
        id_procedimento: "10",
        id_documento: "21",
    }
}

fn aceite_processo(atividade: &str) -> String {
    aceite(
        "procedimento_visualizar",
        "10",
        "procedimento_visualizar_ciencias=1",
        atividade,
    )
}

fn aceite_documento(atividade: &str) -> String {
    aceite(
        "procedimento_visualizar",
        "10",
        "documento_visualizar_ciencias=1&id_documento=21",
        atividade,
    )
}

fn lista(atividade: &str, nome: &str) -> String {
    format!(
        "<html><table id='tblAndamentos' summary='Tabela de Ciências'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='{nome}' value='{atividade}' /></td><td>Unidade1</td><td>aluno1</td></tr></table></html>"
    )
}

#[test]
fn aceite_da_ciencia_do_processo_publica_atividade_vinculada() {
    // Caso de Todos tests/test_ciclo_ciencia_010.py @323eb01
    assert_eq!(
        atividade_do_aceite(&aceite_processo("42457"), processo()),
        Ok("42457".into())
    );
}

#[test]
fn aceite_da_ciencia_do_documento_publica_atividade_vinculada() {
    // Caso de Todos tests/test_documento_ciente_010.py @323eb01
    assert_eq!(
        atividade_do_aceite(&aceite_documento("42458"), documento()),
        Ok("42458".into())
    );
}

#[test]
fn aceite_nao_pode_ser_de_outro_processo_documento_ou_tipo() {
    // Casos de Todos tests/test_documento_ciente_010.py @323eb01
    assert_eq!(
        atividade_do_aceite(&aceite_processo("42"), documento()),
        Err(E::AceiteSemAtividade)
    );
    assert_eq!(
        atividade_do_aceite(&aceite_documento("42"), processo()),
        Err(E::AceiteSemAtividade)
    );
    assert_eq!(
        atividade_do_aceite(
            &aceite(
                "procedimento_visualizar",
                "11",
                "procedimento_visualizar_ciencias=1",
                "42"
            ),
            processo()
        ),
        Err(E::AceiteSemAtividade)
    );
    assert_eq!(
        atividade_do_aceite(
            &aceite(
                "procedimento_visualizar",
                "10",
                "documento_visualizar_ciencias=1&id_documento=22",
                "42"
            ),
            documento()
        ),
        Err(E::AceiteSemAtividade)
    );
}

#[test]
fn aceite_duplicado_com_atividades_distintas_nao_e_prova() {
    // Caso de Todos tests/test_ciclo_ciencia_010.py @323eb01
    let mut html = aceite_processo("42");
    html.push_str(&aceite_processo("43"));
    assert_eq!(
        atividade_do_aceite(&html, processo()),
        Err(E::AtividadesAmbiguas)
    );
}

#[test]
fn aceite_sem_atividade_nao_e_prova() {
    assert_eq!(
        atividade_do_aceite(&aceite_processo(""), processo()),
        Err(E::AceiteSemAtividade)
    );
}

#[test]
fn lista_do_processo_confere_checkbox_da_atividade() {
    // Caso de Todos tests/test_ciclo_ciencia_010.py @323eb01
    assert_eq!(
        atividade_na_lista(&lista("42457", "chkInfraItem"), "42457"),
        Ok(true)
    );
    assert_eq!(
        atividade_na_lista(&lista("42457", "chkInfraItem"), "42458"),
        Ok(false)
    );
}

#[test]
fn lista_da_versao_50_tambem_confere_checkbox() {
    // Caso de Todos tests/test_documento_ciente_010.py @323eb01
    assert_eq!(
        atividade_na_lista(&lista("42458", "chkInfraItem1"), "42458"),
        Ok(true)
    );
}

#[test]
fn checkbox_repetido_na_lista_nao_comprova() {
    let html = lista("42", "chkInfraItem").replace(
        "</td>",
        "<input type='checkbox' name='chkInfraItem2' value='42' /></td>",
    );
    assert_eq!(
        atividade_na_lista(&html, "42"),
        Err(E::AtividadesRepetidasNaLista)
    );
}

#[test]
fn lista_que_nao_e_de_ciencias_nao_comprova() {
    let html = lista("42", "chkInfraItem").replace("Lista de Ciências", "Lista de Andamentos");
    assert_eq!(
        atividade_na_lista(&html, "42"),
        Err(E::ListaDeCienciasForaDaFormaObservada)
    );
}

// RED #271: um aceite relatado só num comentário nunca prova a ciência.
#[test]
fn comentario_js_nao_publica_atividade_da_ciencia() {
    let html = aceite_processo("42457")
        .replace("<script>", "<script>/*")
        .replace("</script>", "*/</script>");
    assert_eq!(
        atividade_do_aceite(&html, processo()),
        Err(E::AceiteSemAtividade)
    );
}

// RED #271: texto citado em JavaScript não é chamada executada.
#[test]
fn literal_js_nao_publica_atividade_da_ciencia() {
    let html = r#"<html><script>
const diagnostico = "atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42457#ID-42457')";
</script></html>"#;
    assert_eq!(
        atividade_do_aceite(html, processo()),
        Err(E::AceiteSemAtividade)
    );
}

// RED #271: um script aninhado em template é inerte.
#[test]
fn script_aninhado_em_template_nao_publica_atividade_da_ciencia() {
    let html = aceite_processo("42457")
        .replace("<script>", "<template><script>")
        .replace("</script>", "</script></template>");
    assert_eq!(
        atividade_do_aceite(&html, processo()),
        Err(E::AceiteSemAtividade)
    );
}

/// RED #271: código que o navegador nunca chama não comprova ciência.
/// A forma legítima observada é a recarga do SEI, não um if(false).
#[test]
fn chamada_em_if_false_nao_comprova_aceite() {
    let chamada = "atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42457#ID-42457');";
    for script in [
        format!("if(false) {{ {chamada} }}"),
        format!("function inicializar(){{ if(false) {{ {chamada} }} }}"),
        format!("if(false) {chamada}"),
        format!("if(false)\\n{chamada}").replace("\\n", "\n"),
        format!("function inicializar(){{ if(false) {chamada} }}"),
    ] {
        let html = format!("<html><script>{script}</script></html>");
        assert_eq!(
            atividade_do_aceite(&html, processo()),
            Err(E::AceiteSemAtividade),
            "ramo inativo não pode publicar atividade: {script}"
        );
    }
}

#[test]
fn chamada_em_funcao_nao_invocada_nao_comprova_aceite() {
    let html = aceite_processo("42457").replace("inicializar", "funcaoNaoInvocada");
    assert_eq!(
        atividade_do_aceite(&html, processo()),
        Err(E::AceiteSemAtividade)
    );
}

/// RED #271: um retorno antecipado ou outra interrupção antes da recarga
/// não pode transformar uma atividade antiga em prova de ciência nova.
#[test]
fn retorno_ou_interrupcao_antes_da_recarga_nao_comprova_aceite() {
    let chamada = "atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42457#ID-42457');";
    for corpo in [
        format!("return; {chamada}"),
        format!("return;\n {chamada}"),
        format!("throw new Error('parou'); {chamada}"),
        format!("if(true) return; {chamada}"),
    ] {
        let html = format!("<html><script>function inicializar(){{ {corpo} }}</script></html>");
        assert_eq!(
            atividade_do_aceite(&html, processo()),
            Err(E::AceiteSemAtividade),
            "código após interrupção não pode provar atividade: {corpo}"
        );
    }
}

// RED #271: a Lista de Ciências pode conter HTML inerte em <template>.
#[test]
fn lista_com_checkbox_no_template_nao_confirma_ciencia() {
    // O SEI precisa publicar a atividade em uma linha real da lista,
    // não apenas num fragmento inerte que o parser DOM percorra.
    let html = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption>
        <tr><td><template><input type='checkbox' name='chkInfraItem' value='42457'></template></td></tr>
        <tr><td><input type='checkbox' name='chkInfraItem2' value='42458'></td></tr>
        </table></html>";
    assert_eq!(
        atividade_na_lista(html, "42457"),
        Err(E::ListaDeCienciasForaDaFormaObservada),
        "checkbox em template não pode provar efeito da escrita"
    );
}


// RED #271: `script` com tipo de dados não executa JavaScript; o texto
// também é inerte quando `src` aponta para um script externo. Esses nós
// não podem provar a atividade publicada pelo aceite de ciência.
#[test]
fn script_de_dados_ou_externo_nao_prova_aceite() {
    let html = aceite_processo("42457");
    for abertura in [
        "<script type='application/json'>",
        "<script type='text/plain'>",
        "<script src='arquivo.js'>",
    ] {
        let alterado = html.replacen("<script>", abertura, 1);
        assert_eq!(
            atividade_do_aceite(&alterado, processo()),
            Err(E::AceiteSemAtividade),
            "script inerte não pode comprovar ciência: {abertura}"
        );
    }
}

#[test]
fn script_javascript_explicito_preserva_o_aceite_observado() {
    let html = aceite_processo("42457").replacen(
        "<script>",
        "<script type='text/javascript'>",
        1,
    );
    assert_eq!(atividade_do_aceite(&html, processo()), Ok("42457".into()));
}
