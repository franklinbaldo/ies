//! Replay de conformidade do acervo público para `ies unidade trocar`
//! (#205): nas cassettes que gravaram `infra_trocar_unidade`, as respostas
//! passam pelos **mesmos leitores** do comando, offline: o link da troca em
//! cada Controle, a tela Trocar Unidade (leitura, escolha e POST derivado),
//! o `Location` do POST e do salto seguinte (a URL que o IES pediria tem de
//! ser a que a gravação pediu) e o Controle da unidade nova. Só contagens,
//! nunca URL. `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
use super::ErroTroca;
use super::retorno::url_do_salto;
use super::tela::{TROCAR_UNIDADE, TelaDeTroca, campos_do_envio, ler_tela_de_troca, link_da_troca};
use crate::cassette::acervo::{Resultado, gravacao, percorrer, publicar, tabela};
use crate::cassette::html_da_resposta;
use crate::controle_reconhecido::unidade_do_controle;
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};

fn classe<T, E>(r: Result<T, E>, sub: impl Fn(&E) -> &'static str) -> Resultado {
    match r {
        Ok(_) => Resultado::Lido,
        Err(e) => Resultado::Recusado(sub(&e)),
    }
}

/// O id que o `Location` declara (o escolhido não está gravado: o corpo do
/// POST não é guardado).
fn id_declarado(localizacao: &str) -> Option<String> {
    let consulta = localizacao.split_once('?')?.1;
    let mut ids = consulta
        .split('&')
        .filter_map(|p| p.strip_prefix("infra_unidade_atual="));
    let id = ids.next()?;
    ids.next().is_none().then(|| id.to_owned())
}

// Um leitor por braço, na ordem da gravação: dividir espalharia o estado.
#[allow(clippy::too_many_lines)]
#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_unidade_trocar_no_acervo() {
    let mut leituras: Vec<(usize, &'static str, String, Resultado)> = Vec::new();
    let lidos = percorrer(&["infra_trocar_unidade"], |_, nome, json| {
        let Some(inst) = gravacao::instalacao(json) else {
            return;
        };
        let rs = gravacao::respostas(json);
        let mut tela: Option<(TelaDeTroca, String)> = None;
        let mut soma =
            |ordem, leitor, r: Resultado| leituras.push((ordem, leitor, nome.to_owned(), r));
        for (i, r) in rs.iter().enumerate() {
            let url = r.interacao["response"]["url"].as_str().unwrap_or("");
            match (r.operacao, r.metodo, r.status) {
                ("procedimento_controlar", "GET", 200) => {
                    let Ok(h) = html_da_resposta(r.interacao) else {
                        continue;
                    };
                    if unidade_do_controle(&h).is_ok() {
                        soma(
                            0,
                            "link da troca no Controle",
                            classe(
                                link_da_troca(&inst, url, &h),
                                crate::erro_publico::rotulo::<ErroTroca>,
                            ),
                        );
                    }
                }
                ("infra_trocar_unidade", "GET", 200) => {
                    let Ok(h) = html_da_resposta(r.interacao) else {
                        continue;
                    };
                    let lida = ler_tela_de_troca(&h);
                    soma(
                        1,
                        "tela Trocar Unidade",
                        classe(lida.as_ref(), |e| crate::erro_publico::Rotulo::rotulo(*e)),
                    );
                    if let Ok(t) = lida {
                        // POST derivado para a primeira unidade não marcada.
                        if let Some(outra) = t.unidades.iter().find(|u| **u != t.marcada) {
                            let servida = TelaServida {
                                url,
                                html: &h,
                                codificacao: CodificacaoDaTela::Iso88591,
                            };
                            let s = derivar_submissao(
                                &TROCAR_UNIDADE,
                                &inst,
                                &servida,
                                &t.marcada.sigla,
                                &campos_do_envio(&t, outra),
                            );
                            soma(
                                2,
                                "POST derivado",
                                classe(
                                    s,
                                    crate::erro_publico::rotulo::<
                                        crate::formulario_web::ErroSubmissaoFormulario,
                                    >,
                                ),
                            );
                        }
                        tela = Some((t, url.to_owned()));
                    }
                }
                ("infra_trocar_unidade", "POST", 302) => {
                    let post = gravacao::url_sem_consulta(json, r.interacao);
                    let loc = gravacao::localizacao(r.interacao).unwrap_or("");
                    let id = id_declarado(loc).unwrap_or_default();
                    let oferecido = tela.as_ref().and_then(|(t, _)| {
                        t.unidades
                            .iter()
                            .find(|u| u.id_unidade == id && **u != t.marcada)
                            .cloned()
                    });
                    soma(
                        3,
                        "id do 302 entre as oferecidas, não a marcada",
                        if oferecido.is_some() {
                            Resultado::Lido
                        } else {
                            Resultado::Recusado("id_fora_da_tela")
                        },
                    );
                    let seguinte = rs
                        .get(i + 1)
                        .map(|s| s.interacao["response"]["url"].as_str().unwrap_or(""));
                    let primeiro = url_do_salto(loc, &post, &id);
                    soma(
                        4,
                        "Location do POST (URL = a pedida)",
                        match (&primeiro, seguinte) {
                            (Ok(u), Some(s)) if u == s => Resultado::Lido,
                            (Ok(_), _) => Resultado::Recusado("url_diferente_da_gravada"),
                            (Err(e), _) => Resultado::Recusado(e.rotulo()),
                        },
                    );
                    let Some(s1) = rs.get(i + 1) else { continue };
                    let mut controle = s1;
                    if s1.status == 302 {
                        let u1 = s1.interacao["response"]["url"].as_str().unwrap_or("");
                        let loc2 = gravacao::localizacao(s1.interacao).unwrap_or("");
                        let s2 = rs.get(i + 2);
                        let segundo = url_do_salto(loc2, u1, &id);
                        soma(
                            5,
                            "Location do 1º salto (URL = a pedida)",
                            match (&segundo, s2) {
                                (Ok(u), Some(s))
                                    if u == s.interacao["response"]["url"]
                                        .as_str()
                                        .unwrap_or("") =>
                                {
                                    Resultado::Lido
                                }
                                (Ok(_), _) => Resultado::Recusado("url_diferente_da_gravada"),
                                (Err(e), _) => Resultado::Recusado(e.rotulo()),
                            },
                        );
                        let Some(s2) = s2 else { continue };
                        controle = s2;
                    }
                    let sigla = oferecido.map(|u| u.sigla).unwrap_or_default();
                    let lida = html_da_resposta(controle.interacao)
                        .ok()
                        .map(|h| unidade_do_controle(&h));
                    soma(
                        6,
                        "Controle da unidade nova",
                        match lida {
                            Some(Ok(u)) if controle.status == 200 && u == sigla => Resultado::Lido,
                            Some(Ok(_)) => {
                                Resultado::Recusado("unidade_do_controle_diferente_da_pedida")
                            }
                            Some(Err(e)) => Resultado::Recusado(e.rotulo()),
                            None => Resultado::NaoAplicavel("corpo ilegível"),
                        },
                    );
                }
                ("infra_trocar_unidade", "POST", _) => soma(
                    4,
                    "Location do POST (URL = a pedida)",
                    Resultado::Recusado("status_inesperado"),
                ),
                _ => {}
            }
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n");
    tabela(
        &mut s,
        leituras.iter().map(|(o, l, n, r)| (*o, *l, n.as_str(), r)),
    );
    publicar(&s);
}
