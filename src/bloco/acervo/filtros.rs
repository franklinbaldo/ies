//! Diagnóstico categórico das listas originais da criação de bloco.
use super::{TipoBloco, gravacao, html_da_resposta, ler_lista};
use crate::bloco::ListaDeBlocos;
use crate::tela_html::{Elemento, HtmlServido, arvore};

pub(super) fn ler_lista_gravada(r: &gravacao::Resposta<'_>) -> (ListaDeBlocos, String) {
    let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).expect("HTML da lista"));
    let lista = ler_lista(&html, TipoBloco::Assinatura)
        .expect("lista de assinatura")
        .lista;
    (lista, observar_filtros(&html))
}

// Observação, não predicado de autoridade. Não publicar valores livres.
fn observar_filtros(html: &HtmlServido) -> String {
    let dom = arvore(html);
    let forms: Vec<_> = dom.com_id("frmBlocoLista").collect();
    let [form] = forms.as_slice() else {
        return "formulário ausente ou não único".to_owned();
    };
    let mut partes = Vec::new();
    for nome in [
        "selGrupoBloco",
        "selUnidadeGeradora",
        "txtPalavrasPesquisaBloco",
        "hdnMeusBlocos",
        "chkSinPrioridade",
        "chkSinRevisao",
        "chkSinComentario",
        "chkSinEstadoGerado",
        "chkSinEstadoDisponibilizado",
        "chkSinEstadoRecebido",
        "chkSinEstadoRetornado",
        "chkSinEstadoConcluido",
    ] {
        let controles: Vec<_> = form
            .descendentes()
            .filter(|no| no.atributo("name") == Some(nome))
            .collect();
        let classe = match controles.as_slice() {
            [] => "ausente",
            [no] if no.presente("disabled") => "desabilitado",
            [no] => classe_do_filtro(*no, nome),
            _ => "repetido",
        };
        partes.push(format!("{nome}={classe}"));
    }
    partes.join("; ")
}

fn classe_do_filtro(no: Elemento<'_>, nome: &str) -> &'static str {
    match no.nome() {
        "select" => {
            let selecionadas: Vec<_> = no
                .elementos("option")
                .filter(|opcao| opcao.presente("selected"))
                .collect();
            match selecionadas.as_slice() {
                [] => "sem selected explícito",
                [opcao] => match opcao.atributo("value") {
                    Some("") => "vazio",
                    Some("null") => "null",
                    Some(_) => "outro",
                    None => "sem value explícito",
                },
                _ => "seleção múltipla",
            }
        }
        "input" if no.atributo("type") == Some("checkbox") => {
            if no.presente("checked") {
                "marcado"
            } else {
                "desmarcado"
            }
        }
        "input" => match no.atributo("value") {
            None => "sem value explícito",
            Some("") => "vazio",
            Some("T") if nome == "hdnMeusBlocos" => "T",
            Some("F") if nome == "hdnMeusBlocos" => "F",
            Some(_) => "outro",
        },
        _ => "tipo não previsto",
    }
}

#[test]
fn diagnostico_dos_filtros_nao_publica_valores_livres() {
    let html = HtmlServido::de_teste(
        "<form id='frmBlocoLista'>\
         <select name='selGrupoBloco'><option selected value='123456'>Grupo</option></select>\
         <input name='txtPalavrasPesquisaBloco' value='texto-reservado'>\
         <input name='hdnMeusBlocos' value='T'>\
         <input name='chkSinEstadoGerado' type='checkbox' checked>\
         <input name='chkSinEstadoConcluido' type='checkbox'></form>",
    );
    let diagnostico = observar_filtros(&html);
    for esperado in [
        "selGrupoBloco=outro",
        "selUnidadeGeradora=ausente",
        "txtPalavrasPesquisaBloco=outro",
        "hdnMeusBlocos=T",
        "chkSinEstadoGerado=marcado",
        "chkSinEstadoConcluido=desmarcado",
    ] {
        assert!(diagnostico.contains(esperado));
    }
    assert!(!diagnostico.contains("123456"));
    assert!(!diagnostico.contains("texto-reservado"));
}
