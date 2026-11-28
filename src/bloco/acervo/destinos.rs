//! Sonda dos corpos originais para #876, sem declarar paridade da CLI.
use crate::cassette::acervo::{gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::tela_html::{HtmlServido, arvore};
use std::collections::BTreeMap;
use std::fmt::Write as _;

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE; sem rede"]
fn replay_diagnostico_dos_destinos_de_bloco() {
    let mut contagens = BTreeMap::new();
    let mut relatorio = String::new();
    percorrer(&["bloco_assinatura_alterar"], |nome, instalacao, json| {
        if !matches!(
            nome,
            "bloco-disponibilizar-pen3--aec2a85f490e" | "bloco-alterar-unb--d36a8c8725eb"
        ) {
            return;
        }
        let mut cadastros = 0;
        let mut consultas = 0;
        let _ = writeln!(relatorio, "{instalacao}:");
        for r in gravacao::respostas(json) {
            if r.operacao == "bloco_assinatura_alterar" && r.metodo == "GET" && r.status == 200 {
                let html = HtmlServido::de_teste(
                    &html_da_resposta(r.interacao).expect("HTML original de alteração"),
                );
                let dom = arvore(&html);
                let mut forms = dom.com_id("frmBlocoCadastro");
                let Some(form) = forms.next() else {
                    continue;
                };
                assert_eq!(form.nome(), "form");
                assert!(forms.next().is_none(), "cadastro duplicado");
                let mut ids = form
                    .elementos("input")
                    .filter(|no| no.atributo("name") == Some("hdnIdBloco"));
                let id = ids
                    .next()
                    .and_then(|no| no.atributo("value"))
                    .expect("identidade oculta do bloco");
                assert!(!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
                assert!(ids.next().is_none());
                let url_ids = gravacao::parametro(r.interacao, "id_bloco");
                if !url_ids.is_empty() {
                    assert!(url_ids.iter().all(|v| *v == id), "identidade divergente");
                }
                let mut selects = form
                    .elementos("select")
                    .filter(|no| no.atributo("name") == Some("selUnidades"));
                let select = selects.next().expect("select de unidades ausente");
                assert!(selects.next().is_none(), "select de unidades duplicado");
                let opcoes = select.elementos("option").count();
                cadastros += 1;
                let _ = writeln!(
                    relatorio,
                    "GET passo {}: opções {opcoes}, identidade URL presente: {}",
                    r.passo,
                    !url_ids.is_empty()
                );
            }
            if r.operacao == "unidade_auto_completar_outras" && r.metodo == "POST" {
                let corpo = gravacao::corpo(r.interacao);
                let mime = gravacao::mime(r.interacao);
                assert_eq!(r.status, 200, "consulta sem resposta 200");
                assert!(!corpo.is_empty(), "resposta consultiva sem corpo");
                consultas += 1;
                let _ = writeln!(
                    relatorio,
                    "POST consultivo passo {}: MIME {mime}, bytes {}",
                    r.passo,
                    corpo.len()
                );
            }
        }
        assert!(cadastros > 0, "nenhum cadastro original localizado");
        assert_eq!(consultas, 2, "censo original de autocomplete");
        assert!(contagens.insert(instalacao.to_owned(), consultas).is_none());
    });
    assert_eq!(contagens.get("pen3"), Some(&2));
    assert_eq!(contagens.get("unb"), Some(&2));
    publicar(&relatorio);
}
