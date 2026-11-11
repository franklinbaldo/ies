//! Replay de conformidade do acervo público para `processo concluir`
//! (Refs #172): nas cassettes que gravaram `procedimento_concluir`, as
//! respostas passam pelos **mesmos leitores** do comando, offline: a ação
//! da barra em cada árvore, `concluirProcesso()` em cada tela do nó, a tela
//! "Conclusão de Processo" (conferência e derivação do POST), o 302 do
//! POST, a resposta da forma `funcao` e, por escrita, a regra de
//! verificação (unidade da tela do processo em `aberto_em` antes e fora
//! dele depois). Só contagens e formas, nunca URL. `#[ignore]`: lê os ZIPs
//! do cache local (`IES_ACERVO_CACHE`).
use super::tela::{
    CAMPOS_DO_ENVIO, CONCLUIR_PROCESSO, conferir_tela_de_conclusao, redirecionamento_esperado,
};
use crate::arvore_processo::{
    AbertoEm, ConcluirNaBarra, ler_arvore, ler_concluir_da_barra, ler_funcao_de_concluir,
    ler_tela_do_processo, nup_declarado_na_arvore,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
use crate::instalacao::Instalacao;
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

/// O que a gravação mostrou até o passo, para a regra de verificação.
#[derive(Default)]
struct Estado {
    unidade: Option<String>,
    /// O último `aberto_em` lido, por id do processo.
    aberto_em: BTreeMap<String, Option<Vec<String>>>,
    /// O id, a unidade e o `aberto_em` de antes da última escrita.
    escrita: Option<(String, Antes)>,
}

type Antes = (Option<String>, Option<Vec<String>>);

impl Estado {
    fn antes(&self, id: &str) -> Antes {
        (
            self.unidade.clone(),
            self.aberto_em.get(id).cloned().flatten(),
        )
    }
}

fn siglas(aberto_em: &AbertoEm) -> Option<Vec<String>> {
    match aberto_em {
        AbertoEm::Unidades(u) => Some(u.iter().map(|u| u.sigla.clone()).collect()),
        _ => None,
    }
}

fn barra(inst: &Instalacao, html: &str, id: &str) -> String {
    match ler_concluir_da_barra(inst, html, id) {
        Ok(Some(ConcluirNaBarra::PelaTela(_))) => "`tela`".into(),
        Ok(Some(ConcluirNaBarra::PelaFuncao { .. })) => "`funcao`".into(),
        Ok(None) => "não oferece".into(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

fn tela_de_conclusao(
    inst: &Instalacao,
    html: &str,
    interacao: &serde_json::Value,
    id: &str,
) -> String {
    if let Err(erro) = conferir_tela_de_conclusao(html, id) {
        return format!("`{erro:?}`");
    }
    let url = interacao["response"]["url"].as_str().unwrap_or("");
    let tela = TelaServida {
        url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    // A unidade não é conferida: a janela não publica `lnkInfraUnidade`.
    match derivar_submissao(&CONCLUIR_PROCESSO, inst, &tela, "", &CAMPOS_DO_ENVIO) {
        Ok(s) if s.nomes_dos_campos().len() == 7 => "conferida e derivada (7 campos)".into(),
        Ok(s) => format!("derivada com {} campos", s.nomes_dos_campos().len()),
        Err(erro) => format!("`{erro:?}`"),
    }
}

fn verificacao(
    antes: &(Option<String>, Option<Vec<String>>),
    depois: Option<&Vec<String>>,
) -> String {
    let (Some(unidade), Some(antes)) = antes else {
        return "unidade ou situação de antes não lida".into();
    };
    if !antes.contains(unidade) {
        return "unidade fora de `aberto_em` antes".into();
    }
    match depois {
        None => "situação de depois não lida".into(),
        Some(d) if d.contains(unidade) => "unidade ainda aberta depois".into(),
        Some(d) if d.is_empty() => "verificada (nenhuma unidade aberta)".into(),
        Some(_) => "verificada (outras unidades seguem abertas)".into(),
    }
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_concluir_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["procedimento_concluir"], |_, nome, json| {
        let Some(inst) = gravacao::instalacao(json) else {
            return;
        };
        let mut estado = Estado::default();
        for r in gravacao::respostas(json) {
            // A tela aberta pela pesquisa rápida traz o id em `id_protocolo`.
            let id = gravacao::id_procedimento(r.interacao)
                .or_else(|| gravacao::id_da_pesquisa_rapida(r.interacao))
                .unwrap_or_default();
            let html = || html_da_resposta(r.interacao).ok();
            match (r.operacao, r.metodo, r.status) {
                ("procedimento_trabalhar", "GET", 200) => {
                    if let Some(tela) =
                        html().and_then(|h| ler_tela_do_processo(&inst, &h, &id, None).ok())
                    {
                        estado.unidade = Some(tela.unidade);
                    }
                }
                ("procedimento_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    somar(&mut contagem, "ação na barra", barra(&inst, &h, &id), nome);
                    let lida = nup_declarado_na_arvore(&h)
                        .and_then(|nup| ler_arvore(&h, &nup, &id).ok())
                        .and_then(|a| siglas(&a.aberto_em));
                    if estado.escrita.as_ref().is_some_and(|(de, _)| *de == id)
                        && let Some((_, antes)) = estado.escrita.take()
                    {
                        let v = verificacao(&antes, lida.as_ref());
                        somar(&mut contagem, "verificação", v, nome);
                    }
                    estado.aberto_em.insert(id.clone(), lida);
                }
                ("arvore_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let classe = match ler_funcao_de_concluir(&inst, &h, &id) {
                        Ok(_) => "URL lida e conferida".to_owned(),
                        Err(erro) => format!("`{erro:?}`"),
                    };
                    somar(&mut contagem, "função na tela do nó", classe, nome);
                }
                ("procedimento_concluir", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    if h.contains("frmDesentranharDocumento") {
                        somar(
                            &mut contagem,
                            "tela de conclusão",
                            tela_de_conclusao(&inst, &h, r.interacao, &id),
                            nome,
                        );
                    } else {
                        let classe = if h.contains("function reabrirProcesso(){") {
                            "HTML 200 (escrita pela função)"
                        } else {
                            "HTML 200 sem a tela do nó"
                        };
                        somar(&mut contagem, "resposta da escrita", classe.into(), nome);
                        estado.escrita = Some((id.clone(), estado.antes(&id)));
                    }
                }
                ("procedimento_concluir", "POST", status) => {
                    let classe = match status {
                        302 if redirecionamento_esperado(
                            gravacao::localizacao(r.interacao),
                            &gravacao::url_sem_consulta(json, r.interacao),
                            &id,
                        ) =>
                        {
                            "302 esperado".to_owned()
                        }
                        302 => "302 não reconhecido".to_owned(),
                        outro => format!("status {outro}"),
                    };
                    somar(&mut contagem, "resposta da escrita", classe, nome);
                    estado.escrita = Some((id.clone(), estado.antes(&id)));
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
