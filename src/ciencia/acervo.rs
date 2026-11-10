//! Replay exclusivo dos cards de ciência de processo/documento (#271).
//! Apenas corpos previamente gravados e ZIPs verificados; NUNCA escreve no SEI.
//! Resultado "lido" significa que o leitor reconheceu a resposta, e não
//! que o efeito da ciência foi comprovado no servidor.
use super::{AlvoDaCiencia, atividade_do_aceite, atividade_na_lista};
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::erro_publico::Rotulo as _;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Registro {
    leitor: &'static str,
    instalacao: String,
    resultado: Resultado,
}

fn id_documento_da_resposta(interacao: &Value) -> Option<String> {
    let ids = gravacao::parametro(interacao, "id_documento");
    let [id] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| (*id).to_owned())
}

/// Prova de replay tem identidade do ALVO, não apenas do processo.
#[derive(Debug)]
enum ProvaPendente {
    Processo {
        id_procedimento: String,
        id_atividade: String,
    },
    Documento {
        id_procedimento: String,
        id_documento: String,
        id_atividade: String,
    },
}

fn conferir_lista(interacao: &Value, atividade: Option<ProvaPendente>) -> Resultado {
    let Ok(html) = html_da_resposta(interacao) else {
        return Resultado::NaoAplicavel("corpo_da_lista_nao_gravado");
    };
    let Some(prova) = atividade else {
        return Resultado::NaoAplicavel("aceite_sem_atividade_observada");
    };
    let Some(processo_da_lista) = gravacao::id_procedimento(interacao) else {
        return Resultado::NaoAplicavel("id_procedimento_da_lista_nao_observado");
    };
    let (processo, id, documento) = match &prova {
        ProvaPendente::Processo {
            id_procedimento,
            id_atividade,
        } => (id_procedimento, id_atividade, None),
        ProvaPendente::Documento {
            id_procedimento,
            id_documento,
            id_atividade,
        } => (id_procedimento, id_atividade, Some(id_documento)),
    };
    if *processo != processo_da_lista {
        return Resultado::NaoAplicavel("lista_de_outro_processo");
    }
    match documento {
        Some(documento) => match id_documento_da_resposta(interacao) {
            Some(id) if id == *documento => {}
            Some(_) => return Resultado::NaoAplicavel("lista_de_outro_documento"),
            None => return Resultado::NaoAplicavel("id_documento_da_lista_nao_observado"),
        },
        None if !gravacao::parametro(interacao, "id_documento").is_empty() => {
            return Resultado::NaoAplicavel("lista_documental_nao_prova_ciencia_do_processo");
        }
        None => {}
    }
    match atividade_na_lista(&html, id) {
        Ok(true) => Resultado::Lido,
        Ok(false) => Resultado::Recusado("atividade_nao_consta_da_lista"),
        Err(erro) => Resultado::Recusado(erro.rotulo()),
    }
}

fn replay(instalacao: &str, json: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    // A prova pendente pertence ao ultimo aceite, nunca a aceites anteriores.
    // A lista deve ainda pertencer ao mesmo processo do aceite.
    let mut atividade: Option<ProvaPendente> = None;
    for resposta in gravacao::respostas(json) {
        if matches!(
            resposta.operacao,
            "procedimento_ciencia" | "documento_ciencia"
        ) {
            atividade = None;
        }
        let (leitor, resultado) = match (resposta.operacao, resposta.metodo, resposta.status) {
            ("procedimento_ciencia" | "documento_ciencia", "GET", 200) => {
                let resultado = match ler_aceite(&resposta) {
                    Ok(prova) => {
                        atividade = Some(prova);
                        Resultado::Lido
                    }
                    Err(resultado) => resultado,
                };
                ("aceite da ciência", resultado)
            }
            ("procedimento_ciencia" | "documento_ciencia", "GET", _) => (
                "aceite da ciência",
                Resultado::NaoAplicavel("status_sem_prova_de_aceite"),
            ),
            ("protocolo_ciencia_listar", "GET", 200) => (
                "lista de ciências",
                conferir_lista(resposta.interacao, atividade.take()),
            ),
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
            .map(|interacao| interacao.expect("card").path())
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
#[ignore = "wip #271: replay externo exige ZIPs públicos verificados; executado no gate dedicado"]
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
mod tests;

/// O GET de aceite gravado: a prova pendente do aceite lido, ou o resultado
/// que o replay registra (não aplicável ou recusado).
fn ler_aceite(resposta: &gravacao::Resposta<'_>) -> Result<ProvaPendente, Resultado> {
    let nao_aplicavel = |motivo| Err(Resultado::NaoAplicavel(motivo));
    // O rótulo da gravação não prova a ação executada: a URL realmente
    // servida deve publicar uma única acao igual à operação. Uma tela
    // preparatória não é GET de ciência.
    if gravacao::parametro(resposta.interacao, "acao").as_slice() != [resposta.operacao] {
        return nao_aplicavel("acao_de_ciencia_nao_observada");
    }
    let Ok(html) = html_da_resposta(resposta.interacao) else {
        return nao_aplicavel("corpo_html_nao_gravado");
    };
    let id_procedimento = gravacao::id_procedimento(resposta.interacao);
    let Some(processo) = id_procedimento.as_deref() else {
        return nao_aplicavel("id_procedimento_nao_observado");
    };
    let id_documento = id_documento_da_resposta(resposta.interacao);
    let alvo = if resposta.operacao == "documento_ciencia" {
        let Some(documento) = id_documento.as_deref() else {
            return nao_aplicavel("id_documento_nao_observado");
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
    let id = atividade_do_aceite(&html, alvo).map_err(|erro| Resultado::Recusado(erro.rotulo()))?;
    Ok(match id_documento {
        Some(id_documento) if resposta.operacao == "documento_ciencia" => {
            ProvaPendente::Documento {
                id_procedimento: processo.to_owned(),
                id_documento,
                id_atividade: id,
            }
        }
        _ => ProvaPendente::Processo {
            id_procedimento: processo.to_owned(),
            id_atividade: id,
        },
    })
}
