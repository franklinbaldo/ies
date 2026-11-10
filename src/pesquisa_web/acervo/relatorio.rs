//! Agregação em Markdown: tabela leitor × instalação, formas lidas por
//! instalação, totais declarados das listas e recusas por erro. Só
//! contagens, formas e `cassette#passo`.
use super::{Registro, Resultado};
use crate::cassette::acervo::INSTALACOES;
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub(super) fn markdown(cassettes: usize, registros: &[Registro]) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "Cassettes lidas: {cassettes}; leituras registradas: {}.\n",
        registros.len()
    );
    crate::cassette::acervo::tabela(
        &mut s,
        registros.iter().map(|r| {
            (
                r.leitor as usize,
                r.leitor.nome(),
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    formas(&mut s, registros);
    totais(&mut s, registros);
    recusas(&mut s, registros);
    s
}

fn coluna(instalacao: &str) -> usize {
    INSTALACOES
        .iter()
        .position(|n| *n == instalacao)
        .unwrap_or(INSTALACOES.len())
}

fn formas(s: &mut String, registros: &[Registro]) {
    let mut contagem: BTreeMap<(usize, &'static str, &'static str), Vec<usize>> = BTreeMap::new();
    for r in registros {
        if let Some(forma) = r.forma {
            let linha = contagem
                .entry((r.leitor as usize, r.leitor.nome(), forma))
                .or_insert_with(|| vec![0; INSTALACOES.len() + 1]);
            linha[coluna(&r.instalacao)] += 1;
        }
    }
    s.push_str("Formas lidas por instalação:\n\n| Leitor | Forma |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push('\n');
    s.push_str("|---|---|");
    s.push_str(&"---:|".repeat(INSTALACOES.len()));
    s.push('\n');
    for ((_, leitor, forma), linha) in &contagem {
        let _ = write!(s, "| {leitor} | `{forma}` |");
        for n in &linha[..INSTALACOES.len()] {
            let _ = write!(s, " {n} |");
        }
        s.push('\n');
    }
    s.push('\n');
}

fn totais(s: &mut String, registros: &[Registro]) {
    s.push_str("Totais declarados das listas lidas (`cassette#passo`):\n\n");
    let mut algum = false;
    for r in registros {
        if let Some(total) = r.total_declarado {
            let _ = write!(s, "- {} `{}#{}`: {total}", r.instalacao, r.card, r.passo);
            if let Some((inicio, fim)) = r.faixa {
                let _ = write!(
                    s,
                    ", faixa {inicio}–{fim}, linhas ausentes {}",
                    r.linhas_ausentes
                );
            }
            s.push('\n');
            algum = true;
        }
    }
    if !algum {
        s.push_str("- nenhuma\n");
    }
    s.push('\n');
}

fn recusas(s: &mut String, registros: &[Registro]) {
    let mut por: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for r in registros {
        if let Resultado::Recusado(sub) = r.resultado {
            por.entry(sub).or_default().push(format!(
                "`{}#{}` ({})",
                r.card,
                r.passo,
                r.leitor.nome()
            ));
        }
    }
    s.push_str("Recusas por erro:\n\n");
    if por.is_empty() {
        s.push_str("- nenhuma\n");
    }
    for (sub, onde) in &por {
        let _ = writeln!(s, "- `{sub}` ({}): {}", onde.len(), onde.join(", "));
    }
}
