//! Replay offline da criação de processos (#245), sem invocar uma escrita.
//! Os ZIPs são conferidos por SHA e esquema. Usamos os leitores da CLI:
//! lista, cadastro e hipótese legal, não um parser paralelo de semântica.
//! Uma resposta POST de criação não comprova efeito sem leitura posterior.
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::processo_criar_cadastro::{conferir_cadastro, conferir_cadastro_com_ajax_pendente};
use crate::processo_criar_hipotese::opcao_unica;
use crate::processo_criar_lista::tipo_na_lista;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Registro {
    leitor: &'static str,
    instalacao: String,
    passo: usize,
    resultado: Resultado,
}

fn nome_da_ancora(html: &str) -> Result<Vec<String>, Resultado> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| Resultado::Recusado("lista_de_tipos_html_invalido".to_owned()))?;
    let tabelas = nos_com_id(&dom, "tblTipoProcedimento")
        .map_err(|_| Resultado::Recusado("lista_de_tipos_html_invalido".to_owned()))?;
    let tabela = match tabelas.as_slice() {
        [] => {
            return Err(Resultado::NaoAplicavel(
                "tabela_de_tipos_ausente".to_owned(),
            ));
        }
        [unica] => unica,
        [_, _, ..] => {
            return Err(Resultado::NaoAplicavel(
                "tabela_de_tipos_repetida".to_owned(),
            ));
        }
    };
    let parser = dom.parser();
    let tag = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(Resultado::Recusado(
            "lista_de_tipos_html_invalido".to_owned(),
        ))?;
    if tag.name().as_utf8_str() != "table" {
        return Err(Resultado::Recusado(
            "tabela_de_tipos_ausente_ou_repetida".to_owned(),
        ));
    }
    // Extrair somente o nome observado. O leitor da CLI valida a ação
    // e a unicidade de cada nome no HTML completo.
    let mut nomes = BTreeSet::new();
    for handle in tag.query_selector(parser, "a").into_iter().flatten() {
        let Some(ancora) = handle.get(parser).and_then(tl::Node::as_tag) else {
            return Err(Resultado::Recusado(
                "lista_de_tipos_html_invalido".to_owned(),
            ));
        };
        let classe = valor_do_atributo(ancora, "class")
            .map_err(|_| Resultado::Recusado("lista_de_tipos_html_invalido".to_owned()))?;
        if !classe
            .as_deref()
            .is_some_and(|s| s.split_whitespace().any(|c| c == "ancoraOpcao"))
        {
            continue;
        }
        let nome = html_escape::decode_html_entities(&ancora.inner_text(parser))
            .into_owned()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !nome.is_empty() {
            nomes.insert(nome);
        }
    }
    Ok(nomes.into_iter().collect())
}

fn ler_lista(html: &str) -> Resultado {
    let nomes = match nome_da_ancora(html) {
        Ok(nomes) => nomes,
        Err(estado) => return estado,
    };
    if nomes.is_empty() {
        return Resultado::NaoAplicavel("sem_tipo_na_lista_gravada".to_owned());
    }
    for nome in nomes {
        if let Err(erro) = tipo_na_lista(html, &nome) {
            return Resultado::Recusado(format!("{erro:?}"));
        }
    }
    Resultado::Lido
}

fn id_tipo_do_cadastro(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let forms = nos_com_id(&dom, "frmProcedimentoCadastro").ok()?;
    let [form] = forms.as_slice() else {
        return None;
    };
    let parser = dom.parser();
    let tag = form.get(parser)?.as_tag()?;
    let mut ids = Vec::new();
    for handle in tag.query_selector(parser, "input").into_iter().flatten() {
        let input = handle.get(parser)?.as_tag()?;
        if valor_do_atributo(input, "name").ok()?.as_deref() == Some("hdnIdTipoProcedimento") {
            ids.push(valor_do_atributo(input, "value").ok()?);
        }
    }
    let [Some(id)] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.clone())
}

fn ler_cadastro(html: &str) -> Resultado {
    let Some(id) = id_tipo_do_cadastro(html) else {
        return Resultado::NaoAplicavel("cadastro_sem_id_tipo_observado".to_owned());
    };
    // A tela pode oferecer dois níveis; basta um nível validado pelos
    // mesmos predicados do comando, sem concluir pela ausência do outro.
    for nivel in ["publico", "restrito"] {
        if conferir_cadastro(html, &id, nivel).is_ok() {
            return Resultado::Lido;
        }
    }
    if conferir_cadastro_com_ajax_pendente(html, &id, "restrito").is_ok() {
        return Resultado::NaoAplicavel("hipotese_restrita_pendente_de_ajax".to_owned());
    }
    // Após a mesma validação ter falhado, não invente uma forma:
    // preserve a causa tipada concreta.
    match conferir_cadastro(html, &id, "publico") {
        Ok(_) => Resultado::Lido,
        Err(erro) => Resultado::Recusado(format!("{erro:?}")),
    }
}

fn ler_xml(interacao: &Value) -> Resultado {
    let mime = gravacao::mime(interacao);
    if !matches!(
        mime.split(';').next().unwrap_or_default().trim(),
        "text/xml" | "application/xml"
    ) {
        return Resultado::NaoAplicavel("resposta_ajax_sem_mime_xml".to_owned());
    }
    let corpo = gravacao::corpo(interacao);
    let codificacao = interacao["response"]["encoding"].as_str().unwrap_or("");
    let Ok(xml) = crate::transporte::corpo::legivel(&corpo, codificacao) else {
        return Resultado::Recusado("hipotese_legal_ajax_resposta_nao_reconhecida".to_owned());
    };
    match opcao_unica(&xml) {
        Ok(_) => Resultado::Lido,
        Err(erro) => Resultado::Recusado(format!("{erro:?}")),
    }
}

fn replay(instalacao: &str, cassette: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    for r in gravacao::respostas(cassette) {
        let (leitor, resultado) = match (r.operacao, r.metodo, r.status) {
            ("procedimento_gerar", "POST", _) => (
                "escrita sem verificação",
                Resultado::NaoAplicavel("post_sem_leitura_verificadora".to_owned()),
            ),
            ("procedimento_escolher_tipo", _, 200) => (
                "lista de tipos",
                match html_da_resposta(r.interacao) {
                    Ok(html) => ler_lista(&html),
                    Err(_) => Resultado::NaoAplicavel("html_da_lista_ilegivel".to_owned()),
                },
            ),
            ("procedimento_gerar", "GET", 200) => (
                "cadastro",
                match html_da_resposta(r.interacao) {
                    Ok(html) => ler_cadastro(&html),
                    Err(_) => Resultado::NaoAplicavel("html_do_cadastro_ilegivel".to_owned()),
                },
            ),
            ("hipotese_legal_select_nome_base_legal", _, 200) => {
                ("hipótese legal AJAX", ler_xml(r.interacao))
            }
            (
                "procedimento_escolher_tipo"
                | "procedimento_gerar"
                | "hipotese_legal_select_nome_base_legal",
                _,
                _,
            ) => (
                "resposta sem leitura",
                Resultado::NaoAplicavel("status_sem_leitor".to_owned()),
            ),
            _ => continue,
        };
        registros.push(Registro {
            leitor,
            instalacao: instalacao.to_owned(),
            passo: r.passo,
            resultado,
        });
    }
    registros
}

/// Somente os cards de processo-criar; não exige ZIPs de outras operações.
/// Todos os ZIPs e membros lidos passam pela conferência compartilhada.
#[test]
#[ignore = "lê os ZIPs de treinamento do acervo em IES_ACERVO_CACHE; sem rede"]
fn replay_de_processo_criar_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let pasta_cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut cards = Vec::new();
    for instalacao in INSTALACOES {
        for entrada in std::fs::read_dir(raiz.join(instalacao)).expect("pasta de cassettes") {
            let caminho = entrada.expect("card").path();
            if caminho
                .file_name()
                .is_some_and(|s| s.to_string_lossy().starts_with("processo-criar-"))
                && caminho.extension().is_some_and(|s| s == "md")
            {
                cards.push(caminho);
            }
        }
    }
    cards.sort();
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    for card in &cards {
        let nome = archive_filename(card).expect("origem do ZIP");
        let zip = zips.entry(nome.clone()).or_insert_with(|| {
            ZipDoAcervo::ler(&pasta_cache.join(&nome))
                .unwrap_or_else(|e| panic!("{nome}: {e} (cache {})", pasta_cache.display()))
        });
        let conferida = conferir_cassette_ate(card, zip, 120_000_000).expect("cassette conferida");
        registros.extend(replay(&conferida.instalacao, &conferida.json));
    }
    let mut s = format!(
        "# Processo criar — replay offline da issue #245\n\n{} cassettes conferidas, {} leituras classificadas. Nenhum POST foi executado.\n\n",
        cards.len(),
        registros.len()
    );
    tabela(
        &mut s,
        registros.iter().map(|r| {
            (
                match r.leitor {
                    "lista de tipos" => 0,
                    "cadastro" => 1,
                    "hipótese legal AJAX" => 2,
                    _ => 3,
                },
                r.leitor,
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    let _ = writeln!(
        s,
        "Passos por instalação: {}.\n",
        registros.iter().map(|r| r.passo + 1).max().unwrap_or(0)
    );
    s.push_str("O corpo da requisição POST pode não existir na gravação; sem releitura correlacionada não se afirma criação.\n");
    publicar(&s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;

    fn passo(operacao: &str, metodo: &str, status: u64, mime: &str, corpo: &str) -> Value {
        json!({
            "request": {"operation": operacao, "method": metodo, "pathname": "/sei/controlador.php"},
            "response": {
                "status_code": status,
                "url": format!("https://sei.treinamento.unb.br/sei/controlador.php?acao={operacao}"),
                "encoding": "utf-8",
                "headers": {"content-type": mime},
                "body_base64": STANDARD.encode(corpo.as_bytes())
            }
        })
    }

    fn lista(links: &str) -> String {
        format!(
            "<button onclick=\"exibirTiposProcedimento('U')\">Tudo</button><table id='tblTipoProcedimento'><tr><td>{links}</td></tr></table>"
        )
    }

    #[test]
    fn replay_reusa_leitores_de_lista_cadastro_e_hipotese() {
        // Cenário estrutural de Todos tests/test_processo_criar_escolher_tipo_010.py @323eb01.
        let tipo = "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>";
        let cadastro = "<form id='frmProcedimentoCadastro'><input name='hdnIdTipoProcedimento' value='12'><input name='txtDescricao' maxlength='250'><input name='rdoNivelAcesso' value='1' type='radio'><select name='selAssuntos'><option value='21'>Assunto</option></select><select name='selHipoteseLegal'><option value='31'>Base legal</option></select></form>";
        let json = json!({"interactions": [
            passo("procedimento_escolher_tipo", "GET", 200, "text/html", &lista(tipo)),
            passo("procedimento_gerar", "GET", 200, "text/html", cadastro),
            passo("hipotese_legal_select_nome_base_legal", "POST", 200, "application/xml", "<select><option value='null'> </option><option value='31'>Base legal</option></select>")
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 3);
        assert_eq!(
            registros.iter().map(|r| &r.resultado).collect::<Vec<_>>(),
            vec![&Resultado::Lido; 3]
        );
    }

    #[test]
    fn replay_nao_confunde_post_com_prova_de_criacao() {
        let json = json!({"interactions": [
            passo("procedimento_gerar", "POST", 302, "text/html", "")
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_eq!(
            registros[0].resultado,
            Resultado::NaoAplicavel("post_sem_leitura_verificadora".to_owned())
        );
    }

    #[test]
    fn replay_detecta_nome_de_tipo_duplicado_pelo_leitor_real() {
        let tipo = "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>";
        let json = json!({"interactions": [
            passo("procedimento_escolher_tipo", "GET", 200, "text/html", &lista(&format!("{tipo}{tipo}")))
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
    }
}
