//! Replay de conformidade do acervo público para as versões de documento
//! (Refs #88): a barra de cada `DOCUMENTO` das árvores gravadas
//! (`procedimento_visualizar`), cada lista `documento_versao_listar` e o
//! rodapé de cada `documento_visualizar` passam pelos **mesmos leitores**
//! do comando, offline. Só contagens e
//! formas, nunca URL. `#[ignore]`: lê os ZIPs de `IES_ACERVO_CACHE`.
use crate::arvore_processo::{
    EdicaoNaBarra, ler_arvore, ler_barra_do_documento, nup_declarado_na_arvore,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::documento_versoes::interpretar_historico;
use crate::rodape_do_documento::versao_do_rodape;
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

fn barras(html: &str, interacao: &serde_json::Value, json: &serde_json::Value) -> Vec<String> {
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
        .map(
            |d| match ler_barra_do_documento(&instalacao, html, &d.id, &id) {
                Ok(barra) => {
                    let lista = if barra.versoes.is_some() {
                        "lista publicada"
                    } else {
                        "sem lista"
                    };
                    let edicao = match barra.edicao {
                        EdicaoNaBarra::NaoAssinado => "`editarConteudo('N')`",
                        EdicaoNaBarra::Assinado => "`editarConteudo('S')`",
                        EdicaoNaBarra::NaoPublicada => "sem edição",
                        EdicaoNaBarra::ForaDaForma => "edição fora da forma",
                    };
                    format!("{lista}, {edicao}")
                }
                Err(erro) => format!("`{erro:?}`"),
            },
        )
        .collect()
}

fn lista(html: &str, interacao: &serde_json::Value) -> String {
    let ids = gravacao::parametro(interacao, "id_documento");
    let Some(id) = ids.first().filter(|p| ids.iter().all(|v| v == *p)) else {
        return "id_documento da gravação ilegível".to_owned();
    };
    match interpretar_historico(html, id, None) {
        Ok(h) => format!(
            "{} versões, atual {}",
            h.len(),
            h.atual()
                .map_or(0, crate::documento_versoes::VersaoDocumento::numero)
        ),
        Err(erro) => format!("`{erro:?}`"),
    }
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_versoes_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "procedimento_visualizar",
        "documento_versao_listar",
        "documento_visualizar",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for r in gravacao::respostas(json) {
            if r.status != 200 {
                continue;
            }
            let Ok(html) = html_da_resposta(r.interacao) else {
                continue;
            };
            match r.operacao {
                "procedimento_visualizar" => {
                    for classe in barras(&html, r.interacao, json) {
                        somar(&mut contagem, "barra do documento", classe, instalacao);
                    }
                }
                "documento_visualizar" => {
                    let classe = match versao_do_rodape(&html) {
                        Some(_) => "rodapé com a versão servida",
                        None => "sem rodapé na forma observada",
                    };
                    somar(&mut contagem, "conteúdo", classe.to_owned(), instalacao);
                }
                "documento_versao_listar" => {
                    let classe = lista(&html, r.interacao);
                    somar(&mut contagem, "lista de versões", classe, instalacao);
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
