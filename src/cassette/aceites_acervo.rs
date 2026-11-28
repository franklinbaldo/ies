//! Gate único dos aceites de escrita: cada 302 que o SEI gravou nas
//! cassettes das sete instalações passa pelo **mesmo predicado** que o
//! comando usa para declarar a escrita aceita. Fixture sintético não entra
//! aqui. O teste falha se:
//!
//! - o IES recusa um 302 de aceite que o SEI deu de fato (o caso do
//!   `atribuir` até a #867: a `UnB` repete `id_procedimento`, e o fixture
//!   inventado tinha um id só);
//! - um POST com 302 de uma ação que o IES conhece (`AcaoDoSei::ler`) não
//!   está classificado: com predicado, confirmado só por releitura ou fora
//!   das escritas do IES. Assim um predicado novo não nasce sem confronto.
//!
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`), sem rede.
//! Roda no CI pelo `scripts/replay acervo`.
use super::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::Redirecionamento302;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// O que a gravação dá a um predicado: o `Location`, a URL do POST sem
/// consulta e os ids decimais da URL do POST (vazios se ausentes).
struct Aceite<'a> {
    localizacao: Option<&'a str>,
    url: &'a str,
    id_procedimento: &'a str,
    id_documento: &'a str,
}

impl Aceite<'_> {
    fn redirecionamento(&self) -> Redirecionamento302 {
        Redirecionamento302::sintetico(self.localizacao, self.url)
    }
}

type Predicado = fn(&Aceite<'_>) -> bool;

/// Operação POST do SEI → predicado de aceite do comando que a envia.
const COM_PREDICADO: &[(&str, &str, Predicado)] = &[
    ("anotacao_registrar", "processo anotar", |a| {
        crate::processo_anotar::redirecionamento_esperado(a.localizacao, a.url, a.id_procedimento)
    }),
    (
        "procedimento_atribuicao_cadastrar",
        "processo atribuir",
        |a| crate::processo_atribuir::aceite_302(&a.redirecionamento(), a.id_procedimento),
    ),
    ("andamento_marcador_cadastrar", "processo marcar", |a| {
        crate::processo_marcar::aceite_302_da_atribuicao(&a.redirecionamento(), a.id_procedimento)
    }),
    (
        "andamento_marcador_remover",
        "processo marcar --retirar",
        |a| crate::processo_marcar::aceite_302_da_remocao(&a.redirecionamento(), a.id_procedimento),
    ),
    ("procedimento_concluir", "processo concluir", |a| {
        crate::processo_concluir::aceite_302(a.localizacao, a.url, a.id_procedimento)
    }),
    ("procedimento_sobrestar", "processo sobrestar", |a| {
        crate::processo_sobrestar::aceite_302(a.localizacao, a.url, a.id_procedimento)
    }),
    ("procedimento_enviar", "processo enviar", |a| {
        crate::processo_enviar::aceite_302(a.localizacao, a.url, a.id_procedimento)
    }),
    ("procedimento_anexar", "processo anexar", |a| {
        a.localizacao
            .is_some_and(|l| crate::processo_anexar::fluxo::aceite_302(l, a.url, a.id_procedimento))
    }),
    (
        "procedimento_excluir_relacionamento",
        "processo relacionar --remover",
        |a| {
            a.localizacao.is_some_and(|l| {
                crate::processo_relacionar::aceite_302_da_remocao(l, a.url, a.id_procedimento)
            })
        },
    ),
    (
        "procedimento_atualizar_andamento",
        "processo andamento",
        |a| {
            crate::processo_andamento_registrar::localizacao_esperada(
                a.localizacao,
                a.url,
                a.id_procedimento,
            )
        },
    ),
    ("comentario_cadastrar", "processo comentar", |a| {
        a.localizacao.is_some_and(|l| {
            crate::processo_comentar_web::id_do_aceite(l, a.url, a.id_procedimento).is_some()
        })
    }),
    ("documento_cancelar", "documento cancelar", |a| {
        a.localizacao.is_some_and(|l| {
            crate::documento_cancelar::aceite_302(l, a.url, a.id_procedimento, a.id_documento)
        })
    }),
    ("bloco_escolher", "bloco incluir --documento", |a| {
        crate::bloco_incluir::redirecionamento_da_inclusao(
            a.localizacao,
            a.url,
            a.id_procedimento,
            a.id_documento,
        )
    }),
];

/// Escritas do IES que não aceitam pelo 302 isolado: a confirmação é a
/// releitura (ou a identidade vem de outra tela). Entram aqui por decisão,
/// não por omissão.
const SO_RELEITURA: &[(&str, &str)] = &[
    (
        "procedimento_gerar",
        "processo criar: reconciliação pelos Gerados",
    ),
    ("documento_gerar", "documento criar: releitura da árvore"),
    (
        "bloco_assinatura_cadastrar",
        "bloco criar: identidade e releitura",
    ),
    (
        "documento_receber",
        "documento incluir-externo: recibo e releitura",
    ),
    (
        "infra_trocar_unidade",
        "unidade trocar: Controle da unidade nova",
    ),
    ("acompanhamento_cadastrar", "processo acompanhar: releitura"),
    (
        "acompanhamento_excluir",
        "processo acompanhar --deixar: releitura",
    ),
    (
        "procedimento_relacionar",
        "processo relacionar: releitura do vínculo",
    ),
    (
        "controle_prazo_excluir",
        "processo prazo --excluir: releitura",
    ),
];

/// Ações que o IES conhece, mas não envia como escrita própria.
const FORA_DAS_ESCRITAS: &[&str] = &[
    "protocolo_pesquisa_rapida",
    "procedimento_gerar_pdf",
    "procedimento_gerar_zip",
    "exibir_arquivo",
    "procedimento_escolher_tipo",
    "documento_escolher_tipo",
    "protocolo_pesquisar",
    // Propriedades: o IES só lê a tela por GET (`propriedades_*`).
    "procedimento_alterar",
    "documento_alterar",
    // Lupa de unidades: só um passo de navegação até a lista.
    "grupo_unidade_cadastrar",
];

fn classe_da_operacao(op: &str) -> Option<&'static str> {
    if let Some((_, comando, _)) = COM_PREDICADO.iter().find(|(o, ..)| *o == op) {
        return Some(comando);
    }
    if SO_RELEITURA.iter().any(|(o, _)| *o == op) || FORA_DAS_ESCRITAS.contains(&op) {
        return Some("");
    }
    None
}

fn decimal_unico(valores: &[&str]) -> String {
    match valores {
        [primeiro, resto @ ..]
            if !primeiro.is_empty()
                && primeiro.bytes().all(|b| b.is_ascii_digit())
                && resto.iter().all(|v| v == primeiro) =>
        {
            (*primeiro).to_owned()
        }
        _ => String::new(),
    }
}

/// (operação, comando, classe) → instalação → contagem.
type Contagem = BTreeMap<(String, &'static str, String), BTreeMap<String, u64>>;

fn replay(json: &Value, instalacao: &str, contagem: &mut Contagem, sem_classe: &mut Contagem) {
    for r in gravacao::respostas(json) {
        if r.metodo != "POST" || r.status != 302 {
            continue;
        }
        let conta = |c: &mut Contagem, comando: &'static str, classe: &str| {
            *c.entry((r.operacao.to_owned(), comando, classe.to_owned()))
                .or_default()
                .entry(instalacao.to_owned())
                .or_default() += 1;
        };
        let Some((_, comando, predicado)) = COM_PREDICADO.iter().find(|(o, ..)| *o == r.operacao)
        else {
            if AcaoDoSei::ler(r.operacao).is_some() && classe_da_operacao(r.operacao).is_none() {
                conta(sem_classe, "", "não classificada");
            }
            continue;
        };
        let url = gravacao::url_sem_consulta(json, r.interacao);
        let id_procedimento = decimal_unico(&gravacao::parametro(r.interacao, "id_procedimento"));
        let id_documento = decimal_unico(&gravacao::parametro(r.interacao, "id_documento"));
        let aceite = Aceite {
            localizacao: gravacao::localizacao(r.interacao),
            url: &url,
            id_procedimento: &id_procedimento,
            id_documento: &id_documento,
        };
        let classe = if predicado(&aceite) {
            "302 aceito"
        } else {
            "302 RECUSADO"
        };
        conta(contagem, comando, classe);
    }
}

fn relatorio(titulo: &str, contagem: &Contagem) -> String {
    let mut s = format!("\n## {titulo}\n\n| Operação | Comando | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((op, comando, classe), por) in contagem {
        let _ = write!(s, "\n| `{op}` | {comando} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    s
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn aceites_de_escrita_no_acervo() {
    let mut contagem = Contagem::new();
    let mut sem_classe = Contagem::new();
    let lidos = percorrer(&[], |_, instalacao, json| {
        replay(json, instalacao, &mut contagem, &mut sem_classe);
    });
    let mut s = format!("{lidos} cassettes lidas.\n");
    s.push_str(&relatorio("Aceites com predicado", &contagem));
    if !sem_classe.is_empty() {
        s.push_str(&relatorio("Ações do IES sem classificação", &sem_classe));
    }
    publicar(&s);

    // Cada comando com predicado precisa de ao menos um 302 gravado.
    for (op, comando, _) in COM_PREDICADO {
        assert!(
            contagem.keys().any(|(o, ..)| o == op),
            "{comando}: nenhum 302 de `{op}` no acervo"
        );
    }
    let recusados: Vec<_> = contagem
        .keys()
        .filter(|(_, _, classe)| classe == "302 RECUSADO")
        .map(|(op, comando, _)| format!("{comando} (`{op}`)"))
        .collect();
    assert!(
        recusados.is_empty(),
        "o IES recusa 302 de aceite gravados pelo SEI: {recusados:?}"
    );
    let soltas: Vec<_> = sem_classe.keys().map(|(op, ..)| op.clone()).collect();
    assert!(
        soltas.is_empty(),
        "ações do IES sem classificação: {soltas:?}"
    );
}
