//! Controle negativo com respostas originais: o editor nativo nao pode
//! satisfazer o seletor exclusivo do ensaio sintetico sei.test.
use super::{editor_observado, nativo};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use std::path::{Path, PathBuf};

// Censo estrutural independente do localizador da foto. Não imprime
// nomes, HTML, identificadores ou URLs. O DOM pode normalizar o primeiro LF
// de textarea, portanto a contagem literal vem do texto original.
fn contar_secoes_classicas(html: &HtmlServido) -> Option<(usize, usize)> {
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmEditor").ok()?;
    let mut total = 0;
    let mut vazias = 0;
    for campo in form
        .elementos("textarea")
        .filter(|c| !c.presente("disabled"))
    {
        let nome = campo.atributo("name")?;
        if !nome.starts_with("txaEditor_") {
            continue;
        }
        let dupla = format!("name=\"{nome}\"");
        let simples = format!("name='{nome}'");
        let mut fontes = html.split("<textarea").skip(1).filter_map(|trecho| {
            let (atributos, resto) = trecho.split_once('>')?;
            if !atributos.contains(&dupla) && !atributos.contains(&simples) {
                return None;
            }
            resto.split_once("</textarea>").map(|(texto, _)| texto)
        });
        let fonte = fontes.next()?;
        if fontes.next().is_some() {
            return None;
        }
        total += 1;
        if fonte.is_empty() {
            vazias += 1;
        }
    }
    (total != 0).then_some((total, vazias))
}

#[test]
fn censo_classico_distingue_vazio_de_quebra_de_linha() {
    let html = HtmlServido::de_teste(
        "<form id='frmEditor'><textarea name='txaEditor_1'></textarea>\
         <textarea name='txaEditor_2'>\n</textarea>\
         <textarea name='txaEditor_3' disabled></textarea></form>",
    );
    assert_eq!(contar_secoes_classicas(&html), Some((2, 1)));
}

fn conferir_recusa_no_original(card_relativo: &str) {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join(card_relativo);
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        PathBuf::from,
    );
    let nome_zip = archive_filename(&card).expect("ZIP do card");
    let zip = ZipDoAcervo::ler(&cache.join(nome_zip)).expect("ZIP original presente");
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("SHA-1 ZIP, SHA-256 membro, schema e origem conferidos");
    let mut observados = 0;
    for resposta in gravacao::respostas(&captura.json) {
        if resposta.operacao != "editor_montar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let pagina = html_da_resposta(resposta.interacao).expect("HTML original legivel");
        let html = HtmlServido::de_teste(&pagina);
        assert!(
            !editor_observado(&html, 1) && !editor_observado(&html, 2),
            "{}: editor_montar no passo {} nao pode ser aceito como sei.test",
            captura.instalacao,
            resposta.passo
        );
        let censo = contar_secoes_classicas(&html);
        if matches!(captura.instalacao.as_str(), "unb" | "ufrj") {
            assert!(censo.is_some(), "formulário clássico original");
            assert_eq!(
                nativo::censo_do_classico(&html),
                censo,
                "{}: leitor nativo diverge do censo independente no passo {}",
                captura.instalacao,
                resposta.passo
            );
        } else if captura.instalacao == "go" {
            assert!(nativo::reconhece_ckeditor(&html));
        }
        if let Some((total, vazias)) = censo {
            println!(
                "editor clássico: instalação={}, passo={}, seções={}, vazias literais={}",
                captura.instalacao, resposta.passo, total, vazias
            );
        } else {
            println!(
                "censo clássico indisponível: instalação={}, passo={}",
                captura.instalacao, resposta.passo
            );
        }
        observados += 1;
    }
    assert_eq!(observados, 3, "tres editores montados na captura original");
}

/// Testa fronteira de escrita em corpos originais, nao replay da CLI inteira.
/// Nao confundir com paridade IES/anti-IES ou live.
#[test]
#[ignore = "requer ZIPs originais conferidos em IES_ACERVO_CACHE"]
fn replay_editor_nativo_no_acervo() {
    conferir_recusa_no_original("cassettes/unb/documento-escritas-unb--36a904749e20.md");
    conferir_recusa_no_original("cassettes/ufrj/documento-escritas-ufrj--2241d2a8f71b.md");
    conferir_recusa_no_original("cassettes/go/documento-escritas-go--16f83179ecaa.md");
}
