//! Replay offline da atribuição (#264), sem enviar POST ou executar JS.
//! Confere ZIPs, membros, schema e host antes de usar o leitor real.
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

struct Registro {
    leitor: &'static str,
    instalacao: String,
    resultado: Resultado,
}

/// Somente um ID oculto singular permite correlacionar a tela com seu
/// formulário; não usamos URL assinada de sessão como identidade pública.
fn id_da_tela(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let forms = nos_com_id(&dom, "frmAtividadeAtribuir").ok()?;
    let [form] = forms.as_slice() else {
        return None;
    };
    let parser = dom.parser();
    let form = form.get(parser)?.as_tag()?;
    let mut ids = Vec::new();
    for node in form.query_selector(parser, "input").into_iter().flatten() {
        let input = node.get(parser)?.as_tag()?;
        if valor_do_atributo(input, "name").ok()?.as_deref() == Some("hdnIdProtocolo") {
            ids.push(valor_do_atributo(input, "value").ok()?);
        }
    }
    let [Some(id)] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.clone())
}

fn replay(instalacao: &str, json: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    for resposta in gravacao::respostas(json) {
        if resposta.operacao != "procedimento_atribuicao_cadastrar" {
            continue;
        }
        let (leitor, resultado) = match (resposta.metodo, resposta.status) {
            ("POST", _) => (
                "resposta de escrita",
                Resultado::NaoAplicavel("post_nao_comprova_atribuicao"),
            ),
            ("GET", 200) => (
                "formulário de atribuição",
                match html_da_resposta(resposta.item) {
                    Err(_) => Resultado::NaoAplicavel("html_nao_observado"),
                    Ok(html) => {
                        let id = gravacao::id_procedimento(resposta.item)
                            .or_else(|| id_da_tela(&html));
                        match id {
                            None => Resultado::NaoAplicavel("id_do_processo_nao_observado"),
                            Some(id) => match super::ler_tela(&html, &id) {
                                Ok(_) => Resultado::Lido,
                                Err(erro) => Resultado::Recusado(erro.subcodigo()),
                            },
                        }
                    }
                },
            ),
            _ => (
                "resposta sem formulário",
                Resultado::NaoAplicavel("status_ou_metodo_nao_lido"),
            ),
        };
        registros.push(Registro {
            leitor,
            instalacao: instalacao.to_owned(),
            resultado,
        });
    }
    registros
}

#[test]
fn replay_sintetico_nao_converte_post_em_prova_de_atribuicao() {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;
    let html = r#"<form id="frmAtividadeAtribuir" method="post" action="controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento=50693&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"><input type="hidden" name="hdnInfraTipoPagina" value="2"><button type="submit" name="sbmSalvar" value="Salvar">Salvar</button><select id="selAtribuicao" name="selAtribuicao"><option value="null" selected> </option><option value="1">aluno2 - Aluno Dois</option></select><input type="hidden" name="hdnIdProtocolo" value="50693"></form>"#;
    let json = json!({"interactions": [
        {
            "request": {"operation": "procedimento_atribuicao_cadastrar", "method": "GET"},
            "response": {"status_code": 200, "url": "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_atribuicao_cadastrar", "encoding": "utf-8", "headers": {"content-type": "text/html"}, "body_base64": STANDARD.encode(html)}
        },
        {
            "request": {"operation": "procedimento_atribuicao_cadastrar", "method": "POST"},
            "response": {"status_code": 302}
        }
    ]});
    let lidos = replay("unb", &json);
    assert_eq!(lidos.len(), 2);
    assert_eq!(lidos[0].resultado, Resultado::Lido);
    assert_eq!(
        lidos[1].resultado,
        Resultado::NaoAplicavel("post_nao_comprova_atribuicao")
    );
}

/// Rodado obrigatoriamente no CI após download dos ZIPs públicos com SHA-1.
/// O test harness normal não possui esses arquivos; não é WIP nem teste
/// desativado, e o job dedicado invoca este teste com --ignored.
#[test]
#[ignore = "acervo #264: executado obrigatoriamente no CI após conferir ZIPs; sem rede"]
fn replay_de_processo_atribuir_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    let mut total_cards = 0;
    for instalacao in INSTALACOES {
        let mut cards = std::fs::read_dir(raiz.join(instalacao))
            .expect("pasta das cassettes")
            .map(|item| item.expect("card").path())
            .filter(|p| p.file_name().is_some_and(|s| s.to_string_lossy().starts_with("processo-atribuir-")) && p.extension().is_some_and(|s| s == "md"))
            .collect::<Vec<_>>();
        cards.sort();
        for card in cards {
            total_cards += 1;
            let nome = archive_filename(&card).expect("origem do ZIP");
            let zip = zips.entry(nome.clone()).or_insert_with(|| {
                ZipDoAcervo::ler(&cache.join(&nome)).unwrap_or_else(|e| panic!("{nome}: {e}"))
            });
            let conferida = conferir_cassette_ate(&card, zip, 120_000_000)
                .expect("hash, schema, completude e host da cassette");
            assert_eq!(conferida.instalacao, instalacao);
            registros.extend(replay(instalacao, &conferida.json));
        }
    }
    assert_eq!(total_cards, INSTALACOES.len(), "uma cassette por instalação");
    assert!(!registros.is_empty(), "nenhuma resposta de atribuição gravada");
    let mut saida = format!(
        "# Atribuição — replay offline #264\n\n{} cassettes, {} respostas classificadas, zero requisições reais. POST histórico não prova escrita.\n\n",
        total_cards, registros.len()
    );
    tabela(
        &mut saida,
        registros.iter().map(|r| {
            (
                if r.leitor == "formulário de atribuição" { 0 } else { 1 },
                r.leitor,
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    let lidos = registros.iter().filter(|r| r.resultado == Resultado::Lido).count();
    let _ = writeln!(saida, "Formulários aceitos pelo leitor real: {lidos}.\n");
    publicar(&saida);
}
