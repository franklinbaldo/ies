#!/usr/bin/env python3
"""Resolver apenas os cinco conflitos conhecidos do merge #459 -> #458.

Uso transitório exclusivo da PR #458; falha fechado se o conjunto mudar.
"""
from pathlib import Path
import subprocess

ESPERADOS = {
    "crates/iesei/src/banco_cassettes.rs",
    "crates/iesei/src/documentos_simulados.rs",
    "crates/iesei/tests/banco_cassettes.rs",
    "crates/iesei/tests/retornos_cassettes.rs",
    "tests/aies_documento_cli.rs",
}

def git(*args):
    subprocess.run(["git", *args], check=True)

def substituir(texto, antes, depois, nome):
    quantidade = texto.count(antes)
    if quantidade != 1:
        raise RuntimeError(f"{nome}: esperado 1 trecho, obtidos {quantidade}: {antes[:80]}")
    return texto.replace(antes, depois)

def editar(caminho, func):
    p = Path(caminho)
    p.write_text(func(p.read_text(encoding="utf-8")), encoding="utf-8")

conflitos = set(subprocess.check_output(
    ["git", "diff", "--name-only", "--diff-filter=U"], text=True
).splitlines())
if conflitos != ESPERADOS:
    raise SystemExit(f"Conflitos mudaram; não conciliar automaticamente: {sorted(conflitos)}")

for caminho in sorted(ESPERADOS - {"tests/aies_documento_cli.rs"}):
    git("checkout", "--theirs", "--", caminho)
git("checkout", "--ours", "--", "tests/aies_documento_cli.rs")

base = "crates/iesei/src/banco_cassettes.rs"
def esquema(s):
    return substituir(
        s,
        '    include_str!("../migrations/0010_sessao_bloqueada.sql")\n);',
        '    include_str!("../migrations/0010_sessao_bloqueada.sql"),\n'
        '    "\\n",\n'
        '    include_str!("../migrations/0011_editor_documento.sql")\n);',
        base,
    )
editar(base, esquema)

editar("crates/iesei/src/documentos_simulados.rs", lambda s: substituir(
    s, '            10\n        );', '            11\n        );', "documentos_simulados"
))
editar("crates/iesei/tests/banco_cassettes.rs", lambda s: substituir(
    s, 'assert_eq!(version, 10);', 'assert_eq!(version, 11);', "banco_cassettes_test"
))
editar("crates/iesei/tests/retornos_cassettes.rs", lambda s: substituir(
    s, 'assert_eq!(versao, 10);', 'assert_eq!(versao, 11);', "retornos_test"
))

origem = "crates/iesei/migrations/0009_editor_documento.sql"
destino = "crates/iesei/migrations/0011_editor_documento.sql"
if any(Path("crates/iesei/migrations").glob("0011_*.sql")):
    raise RuntimeError("Já existe migração 0011; precisa reconciliação nova")
git("mv", origem, destino)
editar(destino, lambda s: substituir(
    substituir(s, "-- 0009:", "-- 0011:", destino),
    "PRAGMA user_version = 9;", "PRAGMA user_version = 11;", destino
))

arquivo = "tests/aies_documento_cli.rs"
def teste(s):
    s = substituir(s,
        "use anti_ies::atender_http;\n",
        "use anti_ies::atender_http;\nuse iesei::acesso_sessao;\nuse iesei::acesso_versao;\n",
        arquivo)
    s = substituir(s,
        "fn estado(caminho: &Path, com_documento: bool, assinaturas: u32) -> (Connection, Instalacao) {",
        "fn estado(caminho: &Path, com_documento: bool, assinaturas: u32, versao_restrita: bool) -> (Connection, Instalacao) {",
        arquivo)
    s = substituir(s,
        "    (banco, instalacao)\n}\n\nfn executar(",
        """    if versao_restrita {
        acesso_versao::restringir(
            &banco, &instalacao, CENARIO, "42", NonZeroU32::new(1).unwrap(),
        ).unwrap();
    }
    (banco, instalacao)
}

fn executar(""",
        arquivo)
    antigo = """fn executar_com_assinaturas(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    assinaturas: u32,
) -> Output {
    let (cert, identidade) = certificado(&pasta.0);"""
    novo = """fn executar_com_assinaturas(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    assinaturas: u32,
) -> Output {
    executar_com_sessao_e_assinaturas(pasta, args, requisicoes, com_documento, false, false, assinaturas)
}

fn executar_com_restricao(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    versao_restrita: bool,
) -> Output {
    executar_com_sessao_e_assinaturas(pasta, args, requisicoes, com_documento, versao_restrita, false, 0)
}

fn executar_com_sessao(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    versao_restrita: bool,
    sessao_bloqueada: bool,
) -> Output {
    executar_com_sessao_e_assinaturas(pasta, args, requisicoes, com_documento, versao_restrita, sessao_bloqueada, 0)
}

fn executar_com_sessao_e_assinaturas(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    versao_restrita: bool,
    sessao_bloqueada: bool,
    assinaturas: u32,
) -> Output {
    let (cert, identidade) = certificado(&pasta.0);"""
    s = substituir(s, antigo, novo, arquivo)
    s = substituir(s,
        "        let (db, instalacao) = estado(&banco, com_documento, assinaturas);",
        """        let (db, instalacao) = estado(&banco, com_documento, assinaturas, versao_restrita);
        if sessao_bloqueada {
            acesso_sessao::bloquear(&db, &instalacao, CENARIO).unwrap();
        }""",
        arquivo)
    if 'mod negativos;' not in s:
        s += '\n#[path = "aies_documento_cli/negativos.rs"]\nmod negativos;\n'
    return s
editar(arquivo, teste)

git("add", *sorted(ESPERADOS), destino, origem)
print("RESOLVIDOS_5: main 0009/0010 + editor 0011; BDD-88 e BDD-89")
