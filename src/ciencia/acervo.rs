//! Replay exclusivo dos cards de ciência de processo/documento (#271).
//! Apenas corpos previamente gravados e ZIPs verificados; NUNCA escreve no SEI.
//! Resultado "lido" significa que o leitor reconheceu a resposta, e não
//! que o efeito da ciência foi comprovado no servidor.
use super::{AlvoDaCiencia, atividade_do_aceite, atividade_na_lista};
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Registro {
    leitor: &'static str,
    instalacao: String,
    resultado: Resultado,
}

fn id_documento_da_resposta(item: &Value) -> Option<String> {
    let ids = gravacao::parametro(item, "id_documento");
    let [id] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| (*id).to_owned())
}

fn replay(instalacao: &str, json: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    // A prova pendente pertence ao ultimo aceite, nunca a aceites anteriores.
    // A lista deve ainda pertencer ao mesmo processo do aceite.
    let mut atividade: Option<(String, String)> = None;
    for resposta in gravacao::respostas(json) {
        if matches!(
            resposta.operacao,
            "procedimento_ciencia" | "documento_ciencia"
        ) {
            atividade = None;
        }
        let (leitor, resultado) = match (resposta.operacao, resposta.metodo, resposta.status) {
            ("procedimento_ciencia" | "documento_ciencia", "GET", 200) => {
                let leitor = "aceite da ciência";
                let Some(html) = html_da_resposta(resposta.item).ok() else {
                    registros.push(Registro {
                        leitor,
                        instalacao: instalacao.into(),
                        resultado: Resultado::NaoAplicavel("corpo_html_nao_gravado"),
                    });
                    continue;
                };
                let id_procedimento = gravacao::id_procedimento(resposta.item);
                let Some(processo) = id_procedimento.as_deref() else {
                    registros.push(Registro {
                        leitor,
                        instalacao: instalacao.into(),
                        resultado: Resultado::NaoAplicavel("id_procedimento_nao_observado"),
                    });
                    continue;
                };
                let alvo = if resposta.operacao == "documento_ciencia" {
                    let id_documento = id_documento_da_resposta(resposta.item);
                    let Some(documento) = id_documento.as_deref() else {
                        registros.push(Registro {
                            leitor,
                            instalacao: instalacao.into(),
                            resultado: Resultado::NaoAplicavel("id_documento_nao_observado"),
                        });
                        continue;
                    };
                    AlvoDaCiencia::Documento {
                        id_procedimento: processo,
                        id_documento: documento,
                    }
                } else {
                    AlvoDaCiencia::Processo {
                        id_procedimento: processo,
                    }
                };
                let resultado = match atividade_do_aceite(&html, alvo) {
                    Ok(id) => {
                        atividade = Some((processo.to_owned(), id));
                        Resultado::Lido
                    }
                    Err(erro) => Resultado::Recusado(erro.subcodigo()),
                };
                (leitor, resultado)
            }
            ("procedimento_ciencia" | "documento_ciencia", "GET", _) => (
                "aceite da ciência",
                Resultado::NaoAplicavel("status_sem_prova_de_aceite"),
            ),
            ("protocolo_ciencia_listar", "GET", 200) => {
                let resultado = match (
                    html_da_resposta(resposta.item),
                    atividade.take(),
                    gravacao::id_procedimento(resposta.item),
                ) {
                    (Ok(html), Some((processo, id)), Some(processo_da_lista))
                        if processo == processo_da_lista =>
                    {
                        match atividade_na_lista(&html, &id) {
                            Ok(true) => Resultado::Lido,
                            Ok(false) => Resultado::Recusado("atividade_nao_consta_da_lista"),
                            Err(erro) => Resultado::Recusado(erro.subcodigo()),
                        }
                    }
                    (Err(_), _, _) => Resultado::NaoAplicavel("corpo_da_lista_nao_gravado"),
                    (_, None, _) => Resultado::NaoAplicavel("aceite_sem_atividade_observada"),
                    (_, Some(_), None) => {
                        Resultado::NaoAplicavel("id_procedimento_da_lista_nao_observado")
                    }
                    (_, Some(_), Some(_)) => Resultado::NaoAplicavel("lista_de_outro_processo"),
                };
                ("lista de ciências", resultado)
            }
            ("protocolo_ciencia_listar", "GET", _) => {
                atividade = None;
                (
                    "lista de ciências",
                    Resultado::NaoAplicavel("status_sem_html_da_lista"),
                )
            }
            _ => continue,
        };
        registros.push(Registro {
            leitor,
            instalacao: instalacao.into(),
            resultado,
        });
    }
    registros
}

fn cards_da_ciencia(raiz: &Path) -> Vec<PathBuf> {
    let mut cards = Vec::new();
    for instalacao in INSTALACOES {
        let pasta = raiz.join(instalacao);
        let encontrados: Vec<_> = std::fs::read_dir(&pasta)
            .expect("pasta do acervo")
            .map(|item| item.expect("card").path())
            .filter(|path| {
                path.file_name().is_some_and(|nome| {
                    let nome = nome.to_string_lossy();
                    (nome.starts_with("processo-ciente-") || nome.starts_with("documento-ciente-"))
                        && nome.ends_with(".md")
                })
            })
            .collect();
        assert_eq!(
            encontrados.len(),
            2,
            "dois cards de ciência esperados na instalação {instalacao}"
        );
        cards.extend(encontrados);
    }
    cards.sort();
    cards
}

#[test]
#[ignore = "replay offline: requer ZIPs públicos verificados em IES_ACERVO_CACHE"]
fn replay_da_ciencia_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let pasta = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let cards = cards_da_ciencia(&raiz);
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    for card in &cards {
        let nome = archive_filename(card).expect("origem verificada");
        let zip = zips.entry(nome.clone()).or_insert_with(|| {
            ZipDoAcervo::ler(&pasta.join(&nome))
                .unwrap_or_else(|erro| panic!("{nome}: {erro} (cache {})", pasta.display()))
        });
        let cassette = conferir_cassette_ate(card, zip, 120_000_000).expect("cassette conferida");
        registros.extend(replay(&cassette.instalacao, &cassette.json));
    }
    assert_eq!(cards.len(), 14, "sete instalações, dois alvos");
    assert!(
        registros.iter().any(|r| r.leitor == "aceite da ciência"),
        "um replay vazio não demonstra o leitor"
    );
    let mut relatorio = format!(
        "# Ciência — replay offline #271\n\n{} cards conferidos; {} respostas classificadas, sem requisições SEI.\n\n",
        cards.len(),
        registros.len()
    );
    tabela(
        &mut relatorio,
        registros.iter().map(|r| {
            (
                usize::from(r.leitor != "aceite da ciência"),
                r.leitor,
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    relatorio.push_str(
        "Lido = resposta reconhecida, NÃO prova do efeito novo. \
        Repetição e identidade do usuário dependem de validação live. \
        302, HTML ausente e IDs não observados permanecem não aplicáveis.\n",
    );
    publicar(&relatorio);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;

    fn resposta_html(operacao: &str, url: &str, html: &str) -> Value {
        json!({
            "request": {"operation": operacao, "method": "GET"},
            "response": {
                "status_code": 200,
                "url": url,
                "headers": {"content-type": "text/html"},
                "encoding": "utf-8",
                "body_base64": STANDARD.encode(html)
            }
        })
    }

    #[test]
    fn replay_de_aceite_e_lista_usa_leitores_de_ambos_alvos() {
        let base = "https://sei.treinamento.unb.br/sei/controlador.php";
        let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
        let casos = [
            (
                "procedimento_ciencia",
                format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>",
            ),
            (
                "documento_ciencia",
                format!("{base}?acao=documento_ciencia&id_procedimento=10&id_documento=21"),
                "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>",
            ),
        ];
        for (operacao, url, aceite) in casos {
            let json = json!({
                "interactions": [
                    resposta_html(operacao, &url, aceite),
                    resposta_html(
                        "protocolo_ciencia_listar",
                        &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
                        lista
                    )
                ]
            });
            let registros = replay("unb", &json);
            assert_eq!(registros.len(), 2);
            assert!(
                registros.iter().all(|r| r.resultado == Resultado::Lido),
                "{operacao}: {registros:?}"
            );
        }
    }

    #[test]
    fn aceite_invalido_de_outro_alvo_nao_reutiliza_atividade_anterior() {
        let base = "https://sei.treinamento.unb.br/sei/controlador.php";
        let valido = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
        let lista_antiga = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
        let json = json!({
            "interactions": [
                resposta_html(
                    "procedimento_ciencia",
                    &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                    valido
                ),
                resposta_html(
                    "procedimento_ciencia",
                    &format!("{base}?acao=procedimento_ciencia&id_procedimento=11"),
                    "<html><p>Sem atividade observada para o novo processo</p></html>"
                ),
                resposta_html(
                    "protocolo_ciencia_listar",
                    &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=11"),
                    lista_antiga
                )
            ]
        });
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 3, "{registros:?}");
        assert_eq!(registros[0].resultado, Resultado::Lido);
        assert!(matches!(registros[1].resultado, Resultado::Recusado(_)));
        assert_eq!(
            registros[2].resultado,
            Resultado::NaoAplicavel("aceite_sem_atividade_observada")
        );
    }

    #[test]
    fn aceite_302_apos_aceite_valido_nao_autoriza_lista_anterior() {
        let base = "https://sei.treinamento.unb.br/sei/controlador.php";
        let valido = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
        let lista_antiga = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
        let json = json!({
            "interactions": [
                resposta_html(
                    "procedimento_ciencia",
                    &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                    valido
                ),
                {
                    "request": {"operation": "documento_ciencia", "method": "GET"},
                    "response": {"status_code": 302}
                },
                resposta_html(
                    "protocolo_ciencia_listar",
                    &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=10"),
                    lista_antiga
                )
            ]
        });
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 3, "{registros:?}");
        assert_eq!(registros[0].resultado, Resultado::Lido);
        assert_eq!(
            registros[1].resultado,
            Resultado::NaoAplicavel("status_sem_prova_de_aceite")
        );
        assert_eq!(
            registros[2].resultado,
            Resultado::NaoAplicavel("aceite_sem_atividade_observada")
        );
    }

    #[test]
    fn lista_de_outro_processo_nao_comprova_aceite_valido() {
        let base = "https://sei.treinamento.unb.br/sei/controlador.php";
        let html = "<html><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');</script></html>";
        let lista = "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='42'></td></tr></table></html>";
        let json = json!({"interactions": [
            resposta_html(
                "procedimento_ciencia",
                &format!("{base}?acao=procedimento_ciencia&id_procedimento=10"),
                html
            ),
            resposta_html(
                "protocolo_ciencia_listar",
                &format!("{base}?acao=protocolo_ciencia_listar&id_procedimento=11"),
                lista
            )
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 2);
        assert_eq!(registros[0].resultado, Resultado::Lido);
        assert_eq!(
            registros[1].resultado,
            Resultado::NaoAplicavel("lista_de_outro_processo")
        );
    }

    #[test]
    fn resposta_302_nao_e_prova_de_aceite() {
        let json = json!({"interactions": [{
            "request": {"operation": "procedimento_ciencia", "method": "GET"},
            "response": {"status_code": 302}
        }]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_eq!(
            registros[0].resultado,
            Resultado::NaoAplicavel("status_sem_prova_de_aceite")
        );
    }

    #[test]
    fn lista_sem_atividade_do_aceite_nao_e_sucesso() {
        let json = json!({"interactions": [{
            "request": {"operation": "protocolo_ciencia_listar", "method": "GET"},
            "response": {"status_code": 200}
        }]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_ne!(registros[0].resultado, Resultado::Lido);
    }
}
