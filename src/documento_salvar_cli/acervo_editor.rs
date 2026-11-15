//! Observacao offline de editores nativos com ZIPs originais e hashes
//! conferidos. Nao exercita POST, a CLI completa nem o anti-IES.
use super::{FormatoNativo, editor_observado, formato_nativo};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::{HtmlServido, arvore};
use std::path::{Path, PathBuf};

#[test]
fn recusa_identificar_editor_sem_protocolo_nativo_observado() {
    let nao_editor =
        HtmlServido::de_teste("<form id='frmEditor'><input name='hdnVersao' value='2'></form>");
    assert_eq!(formato_nativo(&nao_editor), None);
    let configuracao_incompleta =
        HtmlServido::de_teste("<script>window.INFRA_EDITOR_CONFIG = {};</script>");
    assert_eq!(formato_nativo(&configuracao_incompleta), None);
}

fn verificar_original(card_relativo: &str, formato: FormatoNativo) {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join(card_relativo);
    let cache = std::env::var_os("IES_ACERVO_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"));
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
        assert_eq!(
            formato_nativo(&html),
            Some(formato),
            "{}: editor_montar no passo {}",
            captura.instalacao,
            resposta.passo
        );
        assert!(
            !editor_observado(&html, 1) && !editor_observado(&html, 2),
            "editor real nao e o formulario sintetico autorizado a escrever"
        );
        if formato == FormatoNativo::Sei4 {
            let dom = arvore(&html);
            let versoes: Vec<_> = dom
                .elementos("input")
                .filter(|input| input.atributo("name") == Some("hdnVersao"))
                .collect();
            assert_eq!(versoes.len(), 1, "versao precisa ser explicita");
            assert!(
                versoes[0].atributo("value").is_some_and(
                    |versao| !versao.is_empty() && versao.bytes().all(|b| b.is_ascii_digit())
                ),
                "versao da tela precisa ser um numero"
            );
        }
        observados += 1;
    }
    assert_eq!(observados, 3, "tres editores montados na captura original");
}

/// Duas familias comprovadas em corpos originais; protocolo de escrita,
/// transicao IES-anti-IES e live continuam NOT_RUN.
#[test]
#[ignore = "requer ZIPs originais conferidos em IES_ACERVO_CACHE"]
fn replay_editor_nativo_no_acervo() {
    verificar_original(
        "cassettes/unb/documento-escritas-unb--36a904749e20.md",
        FormatoNativo::Sei4,
    );
    verificar_original(
        "cassettes/go/documento-escritas-go--16f83179ecaa.md",
        FormatoNativo::Sei5,
    );
}
