//! Agregação em Markdown: tabela leitor × instalação, páginas publicadas
//! (listas com mais de uma página e o plano da página 2), continuidade
//! entre páginas gravadas, formas vistas e recusas por erro.
use super::continuidade::{Relacao, alem_da_primeira, pares, rotulo};
use super::{Leitor, Registro, Replay, Resultado};
use crate::cassette::acervo::INSTALACOES;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(super) fn markdown(cassettes: usize, replay: &Replay) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "Cassettes lidas: {cassettes}; leituras registradas: {}; páginas de lista lidas: {}.\n",
        replay.registros.len(),
        replay.paginas.len()
    );
    crate::cassette::acervo::tabela(
        &mut s,
        replay.registros.iter().map(|r| {
            (
                r.leitor as usize,
                r.leitor.nome(),
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    somas(&mut s, &replay.registros);
    paginadas(&mut s, &replay.registros);
    continuidade(&mut s, replay);
    formas(&mut s, &replay.registros);
    recusas(&mut s, &replay.registros);
    s
}

fn ordem(instalacao: &str) -> usize {
    INSTALACOES
        .iter()
        .position(|n| *n == instalacao)
        .unwrap_or(INSTALACOES.len())
}

#[derive(Default)]
struct Paginada {
    lidas: usize,
    paginadas: usize,
    maior_total: u64,
    maior_paginas: u64,
    tamanhos: BTreeSet<u64>,
    plano_coerente: usize,
    plano_incoerente: usize,
}

/// Listas lidas com mais de uma página e o plano da página 2: a faixa
/// pedida tem de começar logo depois da exibida.
fn paginadas(s: &mut String, registros: &[Registro]) {
    let mut mapa: BTreeMap<(Leitor, usize), Paginada> = BTreeMap::new();
    let lidos = |l: Leitor| {
        registros
            .iter()
            .filter(move |r| r.leitor == l && r.resultado == Resultado::Lido)
    };
    for (lista, pagina) in [
        (Leitor::Recebidos, Leitor::PaginaRecebidos),
        (Leitor::Gerados, Leitor::PaginaGerados),
        (Leitor::Detalhada, Leitor::PaginaDetalhada),
    ] {
        for r in lidos(lista) {
            let e = mapa.entry((lista, ordem(&r.instalacao))).or_default();
            e.lidas += 1;
            let total = r.contagem("total_declarado").unwrap_or(0);
            if r.contagem("linhas").unwrap_or(0) < total {
                e.paginadas += 1;
                e.maior_total = e.maior_total.max(total);
            }
            let Some(p) = lidos(pagina).find(|p| p.card == r.card && p.passo == r.passo) else {
                continue;
            };
            e.maior_paginas = e.maior_paginas.max(p.contagem("paginas").unwrap_or(0));
            if let Some(inicio) = p.contagem("esperada_inicio") {
                e.tamanhos.insert(p.contagem("tamanho").unwrap_or(0));
                let fim = r.contagem("faixa_fim").or_else(|| r.contagem("linhas"));
                if fim.is_some_and(|f| f + 1 == inicio) {
                    e.plano_coerente += 1;
                } else {
                    e.plano_incoerente += 1;
                }
            }
        }
    }
    s.push_str("| Lista | Instalação | Lidas | Com mais de uma página | Maior total | Maior nº de páginas | Tamanhos | Página 2 logo após a exibida |\n");
    s.push_str("|---|---|---:|---:|---:|---:|---|---|\n");
    for ((leitor, i), e) in &mapa {
        let tamanhos: Vec<String> = e.tamanhos.iter().map(u64::to_string).collect();
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} de {} |",
            leitor.nome(),
            INSTALACOES.get(*i).copied().unwrap_or("?"),
            e.lidas,
            e.paginadas,
            e.maior_total,
            e.maior_paginas,
            tamanhos.join(", "),
            e.plano_coerente,
            e.plano_coerente + e.plano_incoerente
        );
    }
    s.push('\n');
}

/// Soma e máximo de cada contagem dos leitores lidos.
fn somas(s: &mut String, registros: &[Registro]) {
    let mut mapa: BTreeMap<(Leitor, &str), (u64, u64)> = BTreeMap::new();
    for r in registros.iter().filter(|r| r.resultado == Resultado::Lido) {
        for (nome, valor) in &r.contagens {
            let e = mapa.entry((r.leitor, nome)).or_default();
            e.0 += valor;
            e.1 = e.1.max(*valor);
        }
    }
    s.push_str("| Leitor | Contagem | Soma | Máximo |\n|---|---|---:|---:|\n");
    for ((leitor, nome), (soma, maximo)) in &mapa {
        let _ = writeln!(s, "| {} | `{nome}` | {soma} | {maximo} |", leitor.nome());
    }
    s.push('\n');
}

fn continuidade(s: &mut String, replay: &Replay) {
    let alem: Vec<String> = alem_da_primeira(&replay.paginas)
        .map(|p| format!("{}#{} ({})", p.card, p.passo, rotulo(&p.lista)))
        .collect();
    let _ = writeln!(
        s,
        "Páginas gravadas além da primeira (faixa começando depois de 1): {}{}.\n",
        alem.len(),
        if alem.is_empty() {
            String::new()
        } else {
            format!(" — {}", alem.join(", "))
        }
    );
    let mut mapa: BTreeMap<(Relacao, &str, usize), (usize, Vec<String>)> = BTreeMap::new();
    for par in pares(&replay.paginas) {
        let e = mapa
            .entry((par.relacao, rotulo(&par.b.lista), ordem(&par.b.instalacao)))
            .or_default();
        e.0 += 1;
        e.1.push(format!("{}#{}→{}", par.b.card, par.a.passo, par.b.passo));
    }
    s.push_str("| Relação entre páginas consecutivas | Lista | Instalação | Pares | Exemplos |\n");
    s.push_str("|---|---|---|---:|---|\n");
    for ((relacao, lista, i), (n, exemplos)) in &mapa {
        let mostrados: Vec<&str> = exemplos.iter().take(2).map(String::as_str).collect();
        let _ = writeln!(
            s,
            "| {} | {lista} | {} | {n} | {} |",
            relacao.nome(),
            INSTALACOES.get(*i).copied().unwrap_or("?"),
            mostrados.join(", ")
        );
    }
    s.push('\n');
}

fn formas(s: &mut String, registros: &[Registro]) {
    let mut mapa: BTreeMap<(Leitor, &str), (usize, BTreeSet<usize>)> = BTreeMap::new();
    for r in registros {
        for f in &r.formas {
            let e = mapa.entry((r.leitor, f.as_str())).or_default();
            e.0 += 1;
            e.1.insert(ordem(&r.instalacao));
        }
    }
    s.push_str("| Leitor | Forma | Ocorrências | Instalações |\n|---|---|---:|---|\n");
    for ((leitor, forma), (n, inst)) in &mapa {
        let inst: Vec<&str> = inst
            .iter()
            .map(|i| INSTALACOES.get(*i).copied().unwrap_or("?"))
            .collect();
        let _ = writeln!(
            s,
            "| {} | `{forma}` | {n} | {} |",
            leitor.nome(),
            inst.join(", ")
        );
    }
    s.push('\n');
}

fn recusas(s: &mut String, registros: &[Registro]) {
    let mut mapa: BTreeMap<(Leitor, &str, usize), Vec<String>> = BTreeMap::new();
    for r in registros {
        if let Resultado::Recusado(sub) = r.resultado {
            mapa.entry((r.leitor, sub, ordem(&r.instalacao)))
                .or_default()
                .push(format!("{}#{}", r.card, r.passo));
        }
    }
    s.push_str("| Leitor | Erro | Instalação | Leituras | Exemplos (cassette#passo) |\n");
    s.push_str("|---|---|---|---:|---|\n");
    for ((leitor, sub, i), exemplos) in &mapa {
        let mostrados: Vec<&str> = exemplos.iter().take(2).map(String::as_str).collect();
        let _ = writeln!(
            s,
            "| {} | `{sub}` | {} | {} | {} |",
            leitor.nome(),
            INSTALACOES.get(*i).copied().unwrap_or("?"),
            exemplos.len(),
            mostrados.join(", ")
        );
    }
}
