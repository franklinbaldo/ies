//! Replay de conformidade do acervo público para `ies documento DOC`
//! (Refs #60): as respostas gravadas de tela por pesquisa rápida, árvore e
//! `documento_visualizar` passam pelos **mesmos leitores** do comando,
//! offline. Por documento de cada árvore, a forma do `Nos[i].src` por
//! ícone; por conteúdo, título e texto; por `documento_download_anexo`
//! gravado (fatia 3), o leitor do anexo com os cabeçalhos gravados. Só
//! contagens e formas, nunca URL.
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
use crate::arvore_processo::{
    ConteudoNaArvore, ler_arvore, ler_conteudo_na_arvore, ler_tela_do_documento,
    nup_declarado_na_arvore,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::documento_web::{identidade_do_titulo, texto_do_corpo};
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn somar(contagem: &mut Contagem, leitor: &'static str, classe: String, instalacao: &str) {
    *contagem
        .entry((leitor, classe))
        .or_default()
        .entry(instalacao.to_owned())
        .or_default() += 1;
}

fn tela(html: &str, interacao: &serde_json::Value, json: &serde_json::Value) -> Option<String> {
    let id = gravacao::id_da_pesquisa_rapida(interacao)?;
    let instalacao = gravacao::instalacao(json)?;
    Some(match ler_tela_do_documento(&instalacao, html, &id, None) {
        Ok(_) => "documento selecionado na árvore".to_owned(),
        Err(erro) => format!("`{erro:?}`"),
    })
}

fn arvore(html: &str, interacao: &serde_json::Value, json: &serde_json::Value) -> Vec<String> {
    let (Some(id), Some(nup), Some(instalacao)) = (
        gravacao::id_procedimento(interacao),
        nup_declarado_na_arvore(html),
        gravacao::instalacao(json),
    ) else {
        return Vec::new();
    };
    let Ok(lida) = ler_arvore(html, &nup, &id) else {
        return Vec::new();
    };
    lida.documentos
        .iter()
        .map(|d| {
            let icone = d.natureza.icone.as_deref().unwrap_or("?");
            let forma = match ler_conteudo_na_arvore(&instalacao, html, &d.id) {
                Ok(ConteudoNaArvore::Visualizar(_)) => "documento_visualizar".to_owned(),
                Ok(ConteudoNaArvore::DownloadDeAnexo(_)) => "documento_download_anexo".to_owned(),
                Ok(ConteudoNaArvore::Vazio) => "vazio".to_owned(),
                Err(erro) => format!("{erro:?}"),
            };
            format!("`{icone}` → `{forma}`")
        })
        .collect()
}

/// A resposta gravada do anexo pelo leitor do comando, sem `--saida`.
fn anexo(interacao: &serde_json::Value) -> String {
    use crate::anexo_documento::{DestinoDoAnexo, LimiteDoAnexo, ler_anexo};
    use crate::transporte::resposta_binaria::RespostaBinaria;
    let cabecalhos = interacao["response"]["headers"]
        .as_object()
        .map(|mapa| {
            mapa.iter()
                .filter_map(|(n, v)| Some((n.to_ascii_lowercase(), v.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    let resposta = RespostaBinaria {
        status: u16::try_from(interacao["response"]["status_code"].as_u64().unwrap_or(0))
            .unwrap_or(0),
        cabecalhos,
        corpo: std::io::Cursor::new(gravacao::corpo(interacao)),
    };
    match ler_anexo(
        resposta,
        "0",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::SoMetadados,
    ) {
        Ok(lido) => format!(
            "`{}` lido, {}",
            lido.mime,
            if lido.texto.is_some() {
                "com texto"
            } else {
                "sem texto"
            }
        ),
        Err(erro) => format!("`{erro:?}`"),
    }
}

fn conteudo(html: &str) -> String {
    match identidade_do_titulo(html) {
        Err(erro) => format!("`{erro:?}`"),
        Ok(_) => match texto_do_corpo(html) {
            Ok(_) => "título e texto lidos".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        },
    }
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_documento_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "procedimento_trabalhar",
        "procedimento_visualizar",
        "documento_visualizar",
        "documento_download_anexo",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for r in gravacao::respostas(json) {
            if r.operacao == "documento_download_anexo" {
                somar(&mut contagem, "anexo", anexo(r.interacao), instalacao);
                continue;
            }
            if r.status != 200 {
                continue;
            }
            let Ok(html) = html_da_resposta(r.interacao) else {
                continue;
            };
            match r.operacao {
                "procedimento_trabalhar" => {
                    if let Some(classe) = tela(&html, r.interacao, json) {
                        somar(&mut contagem, "tela da pesquisa rápida", classe, instalacao);
                    }
                }
                "procedimento_visualizar" => {
                    for classe in arvore(&html, r.interacao, json) {
                        somar(&mut contagem, "link do conteúdo", classe, instalacao);
                    }
                }
                "documento_visualizar" => {
                    somar(&mut contagem, "conteúdo", conteudo(&html), instalacao);
                }
                _ => {}
            }
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n| Leitor | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((leitor, classe), por) in &contagem {
        let _ = write!(s, "\n| {leitor} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    publicar(&s);
}
