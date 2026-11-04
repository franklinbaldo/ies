//! Agregação dos registros em Markdown: tabela leitor × instalação,
//! maiores árvores e históricos, formas vistas e recusas por subcódigo.
use super::{Leitor, Registro, Resultado};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const INSTALACOES: [&str; 7] = ["unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4"];

#[derive(Default)]
struct Linha {
    total: usize,
    lido: usize,
    recusado: BTreeMap<&'static str, usize>,
    nao_aplicavel: BTreeMap<&'static str, usize>,
}

impl Linha {
    fn somar(&mut self, resultado: &Resultado) {
        self.total += 1;
        match resultado {
            Resultado::Lido => self.lido += 1,
            Resultado::Recusado(s) => *self.recusado.entry(s).or_default() += 1,
            Resultado::NaoAplicavel(m) => *self.nao_aplicavel.entry(m).or_default() += 1,
        }
    }
}

fn detalhe(mapa: &BTreeMap<&'static str, usize>) -> String {
    let total: usize = mapa.values().sum();
    if total == 0 {
        return "0".into();
    }
    let partes: Vec<String> = mapa.iter().map(|(k, v)| format!("`{k}` {v}")).collect();
    format!("{total} ({})", partes.join("; "))
}

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
    let mut linhas: BTreeMap<(Leitor, usize), Linha> = BTreeMap::new();
    for r in registros {
        let i = INSTALACOES
            .iter()
            .position(|n| *n == r.instalacao)
            .unwrap_or(INSTALACOES.len());
        linhas.entry((r.leitor, i)).or_default().somar(&r.resultado);
        linhas
            .entry((r.leitor, INSTALACOES.len() + 1))
            .or_default()
            .somar(&r.resultado);
    }
    s.push_str("| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |\n");
    s.push_str("|---|---|---:|---:|---|---|\n");
    for ((leitor, i), l) in &linhas {
        let instalacao = INSTALACOES.get(*i).copied().unwrap_or("**todas**");
        let _ = writeln!(
            s,
            "| {} | {instalacao} | {} | {} | {} | {} |",
            leitor.nome(),
            l.total,
            l.lido,
            detalhe(&l.recusado),
            detalhe(&l.nao_aplicavel)
        );
    }
    s.push('\n');
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
    s.push_str("| Leitor | Subcódigo | Instalação | Leituras | Exemplos (cassette#passo) |\n");
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
