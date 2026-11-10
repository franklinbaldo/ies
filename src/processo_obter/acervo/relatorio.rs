//! Agregação dos registros em Markdown: tabela leitor × instalação,
//! maiores árvores e históricos, formas vistas e recusas por erro.
use super::{Leitor, Registro, Resultado};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(super) fn markdown(cassettes: usize, registros: &[Registro]) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "Cassettes lidas: {cassettes}; leituras registradas: {}.\n",
        registros.len()
    );
    tabela(&mut s, registros);
    maiores(
        &mut s,
        registros,
        Leitor::Arvore,
        "nos",
        &["documentos", "pastas", "anexados", "movidos", "aguarde"],
    );
    for chave in ["anexados", "movidos", "aguarde"] {
        maiores(
            &mut s,
            registros,
            Leitor::Arvore,
            chave,
            &["nos", "documentos", "pastas"],
        );
    }
    maiores(
        &mut s,
        registros,
        Leitor::Andamentos,
        "linhas",
        &["total_declarado"],
    );
    formas(&mut s, registros);
    recusas(&mut s, registros);
    s
}

fn tabela(s: &mut String, registros: &[Registro]) {
    crate::cassette::acervo::tabela(
        s,
        registros.iter().map(|r| {
            (
                r.leitor as usize,
                r.leitor.nome(),
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
}

fn maiores(s: &mut String, registros: &[Registro], leitor: Leitor, chave: &str, outras: &[&str]) {
    let mut lidos: Vec<&Registro> = registros
        .iter()
        .filter(|r| r.leitor == leitor && r.resultado == Resultado::Lido && r.contagem(chave) > 0)
        .collect();
    lidos.sort_by_key(|r| std::cmp::Reverse(r.contagem(chave)));
    let mut vistos = BTreeSet::new();
    let _ = writeln!(
        s,
        "Maiores leituras de {} por `{chave}` (uma por cassette):\n",
        leitor.nome()
    );
    let _ = writeln!(
        s,
        "| Cassette | Instalação | Passo | {chave} | {} | Formas |",
        outras.join(" | ")
    );
    let _ = writeln!(s, "|---|---|---:|---:|{}---|", "---:|".repeat(outras.len()));
    for r in lidos
        .into_iter()
        .filter(|r| vistos.insert(r.card.clone()))
        .take(8)
    {
        let valores: Vec<String> = outras.iter().map(|o| r.contagem(o).to_string()).collect();
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} |",
            r.card,
            r.instalacao,
            r.passo,
            r.contagem(chave),
            valores.join(" | "),
            r.formas.join(", ")
        );
    }
    s.push('\n');
}

fn formas(s: &mut String, registros: &[Registro]) {
    let mut mapa: BTreeMap<(Leitor, &str), (usize, BTreeSet<&str>)> = BTreeMap::new();
    for r in registros {
        for f in &r.formas {
            let e = mapa.entry((r.leitor, f.as_str())).or_default();
            e.0 += 1;
            e.1.insert(r.instalacao.as_str());
        }
    }
    s.push_str("| Leitor | Forma | Ocorrências | Instalações |\n|---|---|---:|---|\n");
    for ((leitor, forma), (n, inst)) in &mapa {
        let inst: Vec<&str> = inst.iter().copied().collect();
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
    let mut mapa: BTreeMap<(Leitor, &str, &str), Vec<String>> = BTreeMap::new();
    for r in registros {
        if let Resultado::Recusado(sub) = r.resultado {
            mapa.entry((r.leitor, sub, r.instalacao.as_str()))
                .or_default()
                .push(format!("{}#{}", r.card, r.passo));
        }
    }
    s.push_str("| Leitor | Erro | Instalação | Leituras | Exemplos (cassette#passo) |\n");
    s.push_str("|---|---|---|---:|---|\n");
    for ((leitor, sub, inst), exemplos) in &mapa {
        let mostrados: Vec<&str> = exemplos.iter().take(3).map(String::as_str).collect();
        let _ = writeln!(
            s,
            "| {} | `{sub}` | {inst} | {} | {} |",
            leitor.nome(),
            exemplos.len(),
            mostrados.join(", ")
        );
    }
}
