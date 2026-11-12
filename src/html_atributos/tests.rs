//! RED/GREEN do tokenizador: casos reproduzidos do defeito do `tl` 0.7.8 e
//! estados WHATWG de nome e valor de atributo.
use super::*;
use crate::tela_html::HtmlServido;

fn nomes(fonte: &str) -> Vec<String> {
    tokenizar_tag_de_abertura(fonte)
        .unwrap()
        .nomes()
        .map(str::to_owned)
        .collect()
}

fn valor(fonte: &str, nome: &str) -> Option<String> {
    tokenizar_tag_de_abertura(fonte)
        .unwrap()
        .valor(nome)
        .map(str::to_owned)
}

#[test]
fn booleanos_antes_de_name_e_id_nao_perdem_caractere() {
    let fonte = "<select multiple disabled name='x' id='y'>";
    assert_eq!(nomes(fonte), ["multiple", "disabled", "name", "id"]);
    assert_eq!(valor(fonte, "name").as_deref(), Some("x"));
    assert_eq!(valor(fonte, "id").as_deref(), Some("y"));
}

#[test]
fn multiple_seguido_de_id() {
    assert_eq!(nomes("<select multiple id='y'>"), ["multiple", "id"]);
    assert_eq!(
        valor("<select multiple id='y'>", "id").as_deref(),
        Some("y")
    );
}

#[test]
fn disabled_seguido_de_name() {
    let fonte = "<input disabled name='n'>";
    assert_eq!(nomes(fonte), ["disabled", "name"]);
    assert!(
        tokenizar_tag_de_abertura(fonte)
            .unwrap()
            .presente("disabled")
    );
}

#[test]
fn booleano_e_detectado_em_toda_forma_de_valor() {
    for fonte in [
        "<input disabled>",
        "<input disabled=\"\">",
        "<input disabled='disabled'>",
        "<input disabled=disabled>",
        "<input DISABLED />",
        "<input name=a disabled/>",
    ] {
        assert!(
            tokenizar_tag_de_abertura(fonte)
                .unwrap()
                .presente("disabled"),
            "{fonte}"
        );
    }
    assert!(
        !tokenizar_tag_de_abertura("<input name='disabled'>")
            .unwrap()
            .presente("disabled")
    );
}

#[test]
fn valor_entre_aspas_pode_conter_maior_que_e_outra_aspa() {
    let fonte = "<input value=\"a>b 'c'\" title='x\"y>' type=hidden>resto>";
    assert_eq!(valor(fonte, "value").as_deref(), Some("a>b 'c'"));
    assert_eq!(valor(fonte, "title").as_deref(), Some("x\"y>"));
    assert_eq!(valor(fonte, "type").as_deref(), Some("hidden"));
    assert_eq!(nomes(fonte), ["value", "title", "type"]);
}

#[test]
fn sem_aspas_termina_em_espaco_ou_maior_que() {
    let fonte = "<a href=controlador.php?acao=x&amp;y=1 class=c>texto</a>";
    assert_eq!(
        valor(fonte, "href").as_deref(),
        Some("controlador.php?acao=x&amp;y=1")
    );
    assert_eq!(valor(fonte, "class").as_deref(), Some("c"));
}

#[test]
fn igual_com_espacos_opcionais_e_quebras_de_linha() {
    let fonte = "<input\n\tname = 'a'\r\n value =\"b\"  id=\nc>";
    assert_eq!(valor(fonte, "name").as_deref(), Some("a"));
    assert_eq!(valor(fonte, "value").as_deref(), Some("b"));
    assert_eq!(valor(fonte, "id").as_deref(), Some("c"));
}

#[test]
fn nomes_em_maiusculas_viram_minusculos_e_valores_ficam_brutos() {
    let fonte = "<INPUT ID='Z' Name=K Value='&amp;A'>";
    assert_eq!(nomes(fonte), ["id", "name", "value"]);
    assert_eq!(valor(fonte, "id").as_deref(), Some("Z"));
    assert_eq!(valor(fonte, "value").as_deref(), Some("&amp;A"));
}

#[test]
fn sem_igual_e_igual_vazio_sao_distintos() {
    let atributos = tokenizar_tag_de_abertura("<option selected value=''>").unwrap();
    assert_eq!(atributos.valor("selected"), None);
    assert_eq!(atributos.valor("value"), Some(""));
    assert_eq!(atributos.valor("id"), None);
}

#[test]
fn barra_de_autofechamento_nao_vira_atributo() {
    assert_eq!(nomes("<input type='hidden' name=a />"), ["type", "name"]);
    assert_eq!(nomes("<br/>"), Vec::<String>::new());
    assert_eq!(valor("<input name=a/>", "name").as_deref(), Some("a/"));
}

#[test]
fn aspas_coladas_ao_proximo_nome_iniciam_novo_atributo() {
    assert_eq!(nomes("<a id='x'href='y'>"), ["id", "href"]);
    assert_eq!(valor("<a id='x'href='y'>", "href").as_deref(), Some("y"));
}

#[test]
fn repetido_vale_o_primeiro_como_no_navegador() {
    let atributos =
        tokenizar_tag_de_abertura("<input name='a' NAME='b' disabled disabled>").unwrap();
    assert_eq!(atributos.valor("name"), Some("a"));
    assert_eq!(atributos.ocorrencias("name"), 2);
    assert!(atributos.presente("disabled"));
}

#[test]
fn aspas_sem_fechamento_e_erro_tipado() {
    assert_eq!(
        tokenizar_tag_de_abertura("<input name='a>"),
        Err(ErroAtributoHtml::ValorEntreAspasSemFechamento)
    );
    assert_eq!(
        tokenizar_tag_de_abertura("<input name=\"a"),
        Err(ErroAtributoHtml::ValorEntreAspasSemFechamento)
    );
}

#[test]
fn tag_sem_maior_que_ou_sem_inicio_e_erro_tipado() {
    assert_eq!(
        tokenizar_tag_de_abertura("<input name=a"),
        Err(ErroAtributoHtml::FimDaTagDeAberturaAusente)
    );
    assert_eq!(
        tokenizar_tag_de_abertura("<input disabled"),
        Err(ErroAtributoHtml::FimDaTagDeAberturaAusente)
    );
    for fonte in ["input name=a>", "", "<", "< a>", "<>"] {
        assert_eq!(
            tokenizar_tag_de_abertura(fonte),
            Err(ErroAtributoHtml::InicioDaTagDeAberturaAusente),
            "{fonte:?}"
        );
    }
}

#[test]
fn tem_classe_separa_por_espaco_html() {
    let atributos = tokenizar_tag_de_abertura("<a hidden class='x\tprotocoloNormal  y'>").unwrap();
    assert!(atributos.tem_classe("protocoloNormal"));
    assert!(!atributos.tem_classe("protocolo"));
}

#[test]
fn le_a_fonte_da_tag_do_tl_e_nao_o_mapa_corrompido() {
    let html = HtmlServido::de_teste(
        "<form><select multiple disabled name='x' id='y'><option selected value=1>a</option></select></form>",
    );
    let dom = crate::tela_html::interpretar(&html);
    let select = dom
        .query_selector("select")
        .and_then(|mut i| i.next())
        .and_then(|n| n.get(dom.parser()))
        .and_then(tl::Node::as_tag)
        .unwrap();
    assert_eq!(valor_do_atributo(select, "name"), Ok(Some("x".to_owned())));
    assert_eq!(atributo_presente(select, "disabled"), Ok(true));
    assert_eq!(nos_com_id(&dom, "y").unwrap().len(), 1);
    let opcao = dom
        .query_selector("option")
        .and_then(|mut i| i.next())
        .and_then(|n| n.get(dom.parser()))
        .and_then(tl::Node::as_tag)
        .unwrap();
    assert_eq!(atributo_presente(opcao, "selected"), Ok(true));
    assert_eq!(valor_do_atributo(opcao, "value"), Ok(Some("1".to_owned())));
}

#[test]
fn busca_por_id_e_classe_apos_booleano() {
    let html = HtmlServido::de_teste(
        "<div id='raiz'><input disabled id='hdn' value='v'>\
        <a hidden class='protocoloNormal' href='h'>x</a></div><p id='hdn'></p>",
    );
    let servido = html;
    let dom = crate::tela_html::interpretar(&servido);
    assert_eq!(nos_com_id(&dom, "hdn").unwrap().len(), 2);
    let raiz = nos_com_id(&dom, "raiz").unwrap()[0]
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .unwrap();
    let arvore = crate::tela_html::arvore(&servido);
    let raiz_da_arvore = arvore.com_id("raiz").next().unwrap();
    assert_eq!(raiz_da_arvore.com_id("hdn").count(), 1);
    assert_eq!(
        nos_por_elemento_e_classe(&dom, None, "a", "protocoloNormal")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        nos_por_elemento_e_classe(&dom, Some(raiz), "a", "protocoloNormal")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        nos_por_elemento_e_classe(&dom, None, "p", "protocoloNormal").unwrap(),
        []
    );
}
