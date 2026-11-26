//! Replay de conformidade do acervo público para a lista de blocos
//! (`bloco_assinatura_listar`, `bloco_interno_listar`, `bloco_reuniao_listar`):
//! passa as respostas gravadas pelo leitor de produção `ler_lista`, sem
//! afrouxar asserts. Refs #797.
use super::{TipoBloco, ler_lista};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::tela_html::HtmlServido;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn tipo_da_operacao(operacao: &str) -> Option<TipoBloco> {
    match operacao {
        "bloco_assinatura_listar" => Some(TipoBloco::Assinatura),
        "bloco_interno_listar" => Some(TipoBloco::Interno),
        "bloco_reuniao_listar" => Some(TipoBloco::Reuniao),
        _ => None,
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        if r.status != 200 {
            continue;
        }
        let Some(tipo) = tipo_da_operacao(r.operacao) else {
            continue;
        };
        let classe = match html_da_resposta(r.interacao) {
            Ok(html) => match ler_lista(&HtmlServido::de_teste(&html), tipo) {
                Ok(lista) if lista.lista.itens.is_empty() => "vazia".to_owned(),
                Ok(_) => "lida".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            },
            Err(erro) => format!("HTML ilegível: `{erro}`"),
        };
        leituras.push((tipo.rotulo(), classe));
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_da_lista_de_blocos_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "bloco_assinatura_listar",
        "bloco_interno_listar",
        "bloco_reuniao_listar",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for (tipo, classe) in replay(json) {
            *contagem
                .entry((tipo, classe))
                .or_default()
                .entry(instalacao.to_owned())
                .or_default() += 1;
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n| Tipo | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((tipo, classe), por) in &contagem {
        let _ = write!(s, "\n| {tipo} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    s.push('\n');
    publicar(&s);

    let recusas: Vec<_> = contagem
        .keys()
        .filter(|(_, classe)| classe.starts_with('`') || classe.starts_with("HTML"))
        .collect();
    assert!(
        recusas.is_empty(),
        "leitores recusaram a gravação: {recusas:?}"
    );
    let total: u64 = contagem.values().flat_map(|por| por.values()).sum();
    assert!(total > 0, "nenhuma lista de blocos lida");
}

// Primeira fatia #876: confirmar o contrato de criação a partir dos bytes
// originais, não do Location sintético dos testes do Todos.
#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE; sem rede"]
fn replay_da_criacao_de_bloco_de_assinatura_no_acervo() {
    use crate::consulta_do_controlador::ConsultaDoControlador;
    use crate::formulario_web::consulta_do_controlador;
    use crate::tela_html::arvore;

    let mut vistos = BTreeMap::new();
    percorrer(&["bloco_assinatura_cadastrar"], |nome, instalacao, json| {
        let campo = match nome {
            "bloco-ciclo-ufrj--9119d6015d02" => "txtDescricao",
            "bloco-ciclo-pen3--de556c70c2ad" => "txaDescricao",
            _ => return,
        };
        let respostas = gravacao::respostas(json);
        let formulario = respostas
            .iter()
            .find(|r| {
                r.operacao == "bloco_assinatura_cadastrar" && r.metodo == "GET" && r.status == 200
            })
            .expect("GET do cadastro não gravado");
        let html = HtmlServido::de_teste(
            &html_da_resposta(formulario.interacao).expect("HTML do cadastro"),
        );
        let dom = arvore(&html);
        let forms: Vec<_> = dom.com_id("frmBlocoCadastro").collect();
        let [form] = forms.as_slice() else {
            panic!("{instalacao}: cadastro não único");
        };
        assert_eq!(form.nome(), "form");
        let descricoes: Vec<_> = form
            .elementos("textarea")
            .filter_map(|no| no.atributo("name"))
            .filter(|n| matches!(*n, "txtDescricao" | "txaDescricao"))
            .collect();
        assert_eq!(descricoes, [campo], "{instalacao}: descrição");
        assert!(
            form.descendentes()
                .any(|no| no.atributo("name") == Some("sbmCadastrarBloco")),
            "{instalacao}: botão Salvar"
        );

        let post = respostas
            .iter()
            .find(|r| {
                r.passo > formulario.passo
                    && r.operacao == "bloco_assinatura_cadastrar"
                    && r.metodo == "POST"
                    && r.status == 302
            })
            .expect("POST 302 de criação não gravado");
        let url = gravacao::url_sem_consulta(json, post.interacao);
        let consulta = gravacao::localizacao(post.interacao)
            .and_then(|local| consulta_do_controlador(local, &url))
            .and_then(|texto| ConsultaDoControlador::ler(texto).ok())
            .expect("Location original não recuperado");
        assert_eq!(consulta.acao().ok(), Some("bloco_assinatura_listar"));
        let id = consulta.unico("id_bloco").expect("id do bloco no Location");
        assert!(!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));

        let relido = respostas.iter().any(|r| {
            if r.passo <= post.passo
                || r.operacao != "bloco_assinatura_listar"
                || r.metodo != "GET"
                || r.status != 200
            {
                return false;
            }
            let html = html_da_resposta(r.interacao).expect("HTML da lista");
            let lista = ler_lista(&HtmlServido::de_teste(&html), TipoBloco::Assinatura)
                .expect("lista de assinatura");
            lista.lista.itens.iter().any(|b| b.numero == id)
        });
        assert!(
            relido,
            "{instalacao}: identidade não observada após criação"
        );
        *vistos.entry(instalacao.to_owned()).or_insert(0usize) += 1;
    });
    assert_eq!(vistos.get("ufrj"), Some(&1));
    assert_eq!(vistos.get("pen3"), Some(&1));
}
