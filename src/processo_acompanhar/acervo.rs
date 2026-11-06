//! Replay offline de acompanhamento (#254), somente respostas gravadas.
use super::{acompanhamentos_da_lista, grupos_do_cadastro};
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct Registro {
    leitor: &'static str,
    instalacao: String,
    resultado: Resultado,
}

fn replay(instalacao: &str, cassette: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    for r in gravacao::respostas(cassette) {
        let (leitor, resultado) = match (r.operacao, r.metodo, r.status) {
            ("acompanhamento_gerenciar", "GET", 200) => (
                "lista",
                match html_da_resposta(r.item) {
                    Ok(html) => match acompanhamentos_da_lista(&html) {
                        Ok(_) => Resultado::Lido,
                        Err(erro) => Resultado::Recusado(erro.subcodigo()),
                    },
                    Err(_) => Resultado::NaoAplicavel("html_da_lista_nao_gravado"),
                },
            ),
            ("acompanhamento_cadastrar", "GET", 200) => (
                "cadastro",
                match (gravacao::id_procedimento(r.item), html_da_resposta(r.item)) {
                    (Some(id), Ok(html)) => match grupos_do_cadastro(&html, &id) {
                        Ok(_) => Resultado::Lido,
                        Err(erro) => Resultado::Recusado(erro.subcodigo()),
                    },
                    _ => Resultado::NaoAplicavel("cadastro_ou_id_nao_gravado"),
                },
            ),
            ("acompanhamento_gerenciar" | "acompanhamento_cadastrar" | "acompanhamento_excluir", "POST", _) => (
                "POST",
                Resultado::NaoAplicavel("post_nao_comprova_alteracao"),
            ),
            ("acompanhamento_gerenciar" | "acompanhamento_cadastrar" | "acompanhamento_excluir", _, _) => (
                "sem HTML",
                Resultado::NaoAplicavel("resposta_nao_legivel"),
            ),
            _ => continue,
        };
        registros.push(Registro {
            leitor,
            instalacao: instalacao.to_owned(),
            resultado,
        });
    }
    registros
}

/// Cassettes conferidas por SHA-1 do ZIP, SHA-256 do membro, schema e host.
#[test]
#[ignore = "wip #254: ZIPs públicos externos exigem IES_ACERVO_CACHE; CI executa explicitamente"]
fn replay_do_acompanhamento_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut cards = Vec::new();
    for instalacao in INSTALACOES {
        for entrada in std::fs::read_dir(raiz.join(instalacao)).expect("pasta de cassettes") {
            let caminho = entrada.expect("card").path();
            if caminho.file_name().is_some_and(|s| {
                s.to_string_lossy().starts_with("processo-acompanhar-")
            }) && caminho.extension().is_some_and(|s| s == "md") {
                cards.push(caminho);
            }
        }
    }
    cards.sort();
    assert!(cards.len() >= INSTALACOES.len(), "faltam cassettes de acompanhamento");
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    for card in &cards {
        let nome = archive_filename(card).expect("nome do ZIP");
        let zip = zips.entry(nome.clone()).or_insert_with(|| {
            ZipDoAcervo::ler(&cache.join(&nome))
                .unwrap_or_else(|e| panic!("{nome}: {e} ({})", cache.display()))
        });
        let conferida = conferir_cassette_ate(card, zip, 120_000_000)
            .unwrap_or_else(|e| panic!("{}: {e}", card.display()));
        registros.extend(replay(&conferida.instalacao, &conferida.json));
    }
    assert!(registros.iter().any(|r| r.leitor == "lista"), "sem listas gravadas");
    assert!(registros.iter().any(|r| r.leitor == "cadastro"), "sem cadastros gravados");
    let mut relatorio = format!(
        "# Acompanhamento Especial — replay #254\n\n{} cassettes conferidas; {} respostas classificadas. Zero requisições ao SEI e zero escritas.\n\n",
        cards.len(),
        registros.len()
    );
    tabela(
        &mut relatorio,
        registros.iter().map(|r| {
            (
                match r.leitor {
                    "lista" => 0,
                    "cadastro" => 1,
                    "POST" => 2,
                    _ => 3,
                },
                r.leitor,
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    relatorio.push_str("Um POST na cassette não prova efeito sem releitura correlacionada.\n");
    publicar(&relatorio);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;

    fn passo(operacao: &str, metodo: &str, status: u64, url: &str, html: &str) -> Value {
        json!({
            "request": {"operation": operacao, "method": metodo},
            "response": {
                "status_code": status,
                "url": url,
                "encoding": "utf-8",
                "headers": {"content-type": "text/html"},
                "body_base64": STANDARD.encode(html.as_bytes())
            }
        })
    }

    #[test]
    fn html_sem_tabela_nao_comprova_ausencia_de_acompanhamento() {
        let dados = json!({"interactions": [
            passo("acompanhamento_gerenciar", "GET", 200,
                "https://sei.treinamento.unb.br/sei/controlador.php?acao=acompanhamento_gerenciar",
                "<html>sem tabela</html>")
        ]});
        let resultado = replay("unb", &dados);
        assert_eq!(resultado.len(), 1);
        assert_eq!(
            resultado[0].resultado,
            Resultado::Recusado("lista_de_acompanhamentos_nao_publicada")
        );
    }

    #[test]
    fn post_gravado_nao_e_prova_de_sucesso() {
        let dados = json!({"interactions": [
            passo("acompanhamento_excluir", "POST", 302,
                "https://sei.treinamento.unb.br/sei/controlador.php?acao=acompanhamento_excluir",
                "")
        ]});
        let resultado = replay("unb", &dados);
        assert_eq!(resultado.len(), 1);
        assert_eq!(
            resultado[0].resultado,
            Resultado::NaoAplicavel("post_nao_comprova_alteracao")
        );
    }
}
