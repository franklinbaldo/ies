//! Censo da forma original, sem enviar cadastro ou imprimir dados da captura.
use super::*;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};

#[test]
#[ignore = "ZIPs originais no IES_ACERVO_CACHE; offline"]
fn cassette_previa_criacao_marcador_unb_go() {
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    for (instancia, caminho) in [
        (
            "unb",
            "cassettes/unb/marcador-ciclo-completo-unb--b24fe5a425ac.md",
        ),
        (
            "go",
            "cassettes/go/marcador-ciclo-completo-go--ee16e8e455ef.md",
        ),
    ] {
        let card = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(caminho);
        let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
        let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        let instalacao = gravacao::instalacao(&captura.json).unwrap();
        let mut formularios = 0;
        let mut reconhecidos = 0;
        let mut listas_mistas = 0;
        let mut encadeadas = 0;
        let mut anterior: Option<String> = None;
        for r in gravacao::respostas(&captura.json) {
            let link = anterior.take();
            if r.metodo != "GET" || r.status != 200 {
                continue;
            }
            match r.operacao {
                "marcador_listar" => {
                    let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                    let servida = gravacao::url_sem_consulta(&captura.json, r.interacao);
                    if marcadores_unidade::ler(&html).is_ok() {
                        listas_mistas += 1;
                        anterior = link_novo(&html, &servida, &instalacao).ok();
                    }
                }
                "marcador_cadastrar" => {
                    formularios += 1;
                    let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
                    let servida = gravacao::url_sem_consulta(&captura.json, r.interacao);
                    if cor_do_formulario(&html, &servida, &instalacao, "Preto")
                        .is_ok_and(|codigo| codigo == "0")
                    {
                        reconhecidos += 1;
                        if link.as_deref() == r.interacao["response"]["url"].as_str() {
                            encadeadas += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        println!(
            "{instancia}: GET cadastro={formularios}, forma reconhecida={reconhecidos}, \
             listas mistas={listas_mistas}, links consecutivos={encadeadas}"
        );
        assert!(formularios > 0, "{instancia}: nenhum GET de cadastro");
        // O HTML versionado do Todos é a testemunha UnB. A família GO pode
        // divergir: este censo não atribui cobertura a formas não reconhecidas.
        if instancia == "unb" {
            assert!(reconhecidos > 0, "UnB: nenhuma forma original reconhecida");
        }
    }
}
