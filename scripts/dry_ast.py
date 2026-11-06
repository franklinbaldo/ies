"""Auditoria DRY global e codemod conservador com Tree-sitter Rust.

Uso:
  uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 python scripts/dry_ast.py --audit
  uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 python scripts/dry_ast.py --apply
  uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 python scripts/dry_ast.py --check

Percorre todos os arquivos Rust em src/. Somente a emissão JSON repetida,
com sintaxe reconhecida, é reescrita. Clones de fluxos com efeitos remotos
NUNCA são fundidos sem comprovação específica de equivalência semântica.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re

from tree_sitter import Language, Parser
import tree_sitter_rust

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
REPORT = ROOT / "docs/experimentos/auditoria-dry-ast-2026-11-05.md"
PARSER = Parser(Language(tree_sitter_rust.language()))
PRETTY = re.compile(rb"serde_json\s*::\s*to_string_pretty\s*\(")
SUFFIX = re.compile(rb"\s*\.\s*(?:unwrap_or_default|unwrap)\s*\(\s*\)", re.S)
FALLBACK = re.compile(rb'\s*\.\s*unwrap_or_else\s*\(\s*\|_\|\s*"\{\}"\s*\.to_owned\(\s*\)\s*\)', re.S)


def nodes(root):
    stack = [root]
    while stack:
        n = stack.pop()
        yield n
        stack.extend(reversed(n.children))


def code(n, source):
    return source[n.start_byte:n.end_byte]


def in_test(path, node, source):
    if "tests" in path.parts or path.name.startswith(("test", "acervo")):
        return True
    cur = node
    while cur is not None:
        if cur.type in ("function_item", "mod_item"):
            sibling = cur.prev_named_sibling
            while sibling is not None and sibling.type == "attribute_item":
                if code(sibling, source) in (b"#[test]", b"#[cfg(test)]"):
                    return True
                sibling = sibling.prev_named_sibling
        cur = cur.parent
    return False


def closing_paren(source, start):
    depth = 0
    quoted = escaped = False
    for i in range(start, len(source)):
        ch = source[i]
        if quoted:
            if escaped:
                escaped = False
            elif ch == 92:
                escaped = True
            elif ch == 34:
                quoted = False
        elif ch == 34:
            quoted = True
        elif ch == 40:
            depth += 1
        elif ch == 41:
            depth -= 1
            if not depth:
                return i + 1
    return None


def valid_expr(source):
    return not PARSER.parse(b"fn _probe() { let _ = " + source + b"; }").root_node.has_error


def find_pretty(source, start, end):
    matches = []
    for found in PRETTY.finditer(source, start, end):
        stop = closing_paren(source, found.end() - 1)
        if stop is None or stop > end:
            continue
        suffix = SUFFIX.match(source, stop) or FALLBACK.match(source, stop)
        if suffix is None or suffix.end() > end:
            continue
        arg = source[found.end():stop - 1].strip()
        expression = source[found.start():suffix.end()]
        replacement = b"crate::saida::json_pretty(" + arg + b")"
        if arg.startswith(b"&") and valid_expr(expression) and valid_expr(replacement):
            matches.append((found.start(), suffix.end(), replacement))
    return matches


def transformations(path, source, root):
    """Macro println!("{}", to_string_pretty(...).unwrap*) -> writer direto.

    Conserva apenas chamadas totalmente reconhecidas por AST, nunca
    reescreve prints parciais, formatos diferentes ou blocos de teste.
    """
    selected = []
    for n in nodes(root):
        if n.type != "macro_invocation" or in_test(path, n, source):
            continue
        macro = n.child_by_field_name("macro")
        if macro is None or code(macro, source) != b"println":
            continue
        tree = next((c for c in n.children if c.type == "token_tree"), None)
        if tree is None:
            continue
        candidates = find_pretty(source, tree.start_byte, tree.end_byte)
        if len(candidates) != 1:
            continue
        a, b, replacement = candidates[0]
        prefix = source[tree.start_byte + 1:a].strip()
        suffix = source[b:tree.end_byte - 1].strip()
        if prefix != b'"{}",' or suffix not in (b"", b","):
            continue
        replacement = replacement.replace(b"json_pretty", b"imprimir_json", 1)
        recuo = source[source.rfind(b"\n", 0, n.start_byte) + 1:n.start_byte]
        statement = (
            b"if let Err(erro) = " + replacement + b" {\n"
            + recuo + b'    eprintln!("IES: falha ao emitir JSON: {erro}");\n'
            + recuo + b"    return std::process::ExitCode::from(1);\n"
            + recuo + b"}"
        )
        selected.append((n.start_byte, n.end_byte, statement))
    return selected


def tokens(body, source):
    return [code(n, source) for n in nodes(body) if not n.children
            and n.type not in ("line_comment", "block_comment")
            and code(n, source).strip()]


def audit():
    metrics = defaultdict(int)
    exact, shape = defaultdict(list), defaultdict(list)
    files = []
    for path in sorted(SRC.rglob("*.rs")):
        source = path.read_bytes()
        tree = PARSER.parse(source)
        if tree.root_node.has_error:
            metrics["arquivos_ast_parcial"] += 1
            # Não reescrever fonte cuja gramática não interpretou integralmente.
            files.append((path, source, []))
            continue
        metrics["arquivos_rust"] += 1
        changes = transformations(path, source, tree.root_node)
        metrics["serializacoes_json_repetidas"] += len(changes)
        files.append((path, source, changes))
        for n in nodes(tree.root_node):
            if in_test(path, n, source):
                continue
            if n.type == "function_item":
                body = n.child_by_field_name("body")
                if body is None:
                    continue
                tok = tokens(body, source)
                if len(tok) < 45:
                    continue
                metrics["funcoes_inspecionadas"] += 1
                loc = f"{path.relative_to(ROOT)}:{n.start_point.row + 1}"
                key = hashlib.sha256(b"\0".join(tok)).hexdigest()
                exact[key].append((loc, len(tok)))
                skey = hashlib.sha256(b"\0".join(
                    b"IDENT" if re.fullmatch(rb"[A-Za-z_]\w*", x) else x for x in tok
                )).hexdigest()
                shape[skey].append((loc, len(tok)))
            elif n.type == "call_expression":
                fn = n.child_by_field_name("function")
                if fn is not None:
                    f = code(fn, source)
                    if f == b"Instalacao::interpretar":
                        metrics["validacoes_instalacao"] += 1
                    if f.endswith(b"map_err"):
                        metrics["chamadas_map_err"] += 1
            elif n.type == "field_expression":
                fld = n.child_by_field_name("field")
                if fld is not None and code(fld, source) == b"unwrap_or_default":
                    metrics["unwrap_or_default"] += 1
    exact = sorted((g for g in exact.values() if len(g) > 1),
                   key=lambda g: (-len(g), -g[0][1]))
    shape = sorted((g for g in shape.values() if len(g) > 1),
                   key=lambda g: (-len(g), -g[0][1]))
    metrics["grupos_clones_exatos"] = len(exact)
    metrics["grupos_clones_forma"] = len(shape)
    return dict(metrics), files, exact, shape


def helper(source):
    if b"pub(crate) fn imprimir_json(" in source:
        return source
    anchor = b"use serde_json::{Value, json};\n"
    if source.count(anchor) != 1:
        raise ValueError("src/saida.rs: ancora de import nao encontrada")
    implementation = (
        b"\n/// Saida JSON da CLI, sem Value->String intermedio; o dominio continua tipado.\n"
        b"pub(crate) fn imprimir_json<T: Serialize>(valor: &T) -> std::io::Result<()> {\n"
        b"    use std::io::Write as _;\n"
        b"    let saida = std::io::stdout();\n"
        b"    let mut bloqueada = saida.lock();\n"
        b"    serde_json::to_writer_pretty(&mut bloqueada, valor).map_err(std::io::Error::from)?;\n"
        b'    bloqueada.write_all(b"\\n")?;\n'
        b"    Ok(())\n"
        b"}\n"
    )
    return source.replace(anchor, anchor + implementation)


def report(metrics, exact, shape, changed):
    lines = [
        "---", "type: Guide",
        "title: Auditoria DRY da codebase por AST",
        "description: Inventario de duplicacoes e normalizacao segura da apresentacao JSON",
        "---", "", "# Auditoria DRY da codebase Rust", "",
        "Tree-sitter percorreu todos os fontes Rust em src/. Apenas a emissao JSON",
        "foi unificada por escrita direta no stdout; equivalencia textual nao prova equivalencia de",
        "dominios, e as regras de escrita no SEI permanecem separadas.", "",
        "## Metricas (antes do codemod)", "",
    ]
    lines.extend(f"- {k}: {v}" for k, v in sorted(metrics.items()))
    lines.extend(["", "## Transformacoes AST aplicadas", ""])
    lines.extend(f"- {path}: {count} chamadas" for path, count in changed)
    lines.extend(["", "## Funcoes identicas (candidatas a revisao)", ""])
    lines.extend(f"- {len(g)} ocorrencias, {g[0][1]} tokens: " +
                 ", ".join(loc for loc, _ in g[:7]) for g in exact[:15])
    lines.extend(["", "## Funcoes com estrutura semelhante (revisao manual)", ""])
    lines.extend(f"- {len(g)} ocorrencias, {g[0][1]} tokens: " +
                 ", ".join(loc for loc, _ in g[:6]) for g in shape[:15])
    lines.extend(["", "## Proximas intervencoes", "",
        "- Retirar revalidacao repetida de sessao, preservando fronteiras de seguranca.",
        "- Corrigir descarte de erros em trailers HTTP chunked.",
        "- Desduplicar validadores apenas quando representam a mesma regra.",
        "- Corrigir tipos de erro reutilizados por pontos de origem distintos (issue #299).",
        "- Nao transformar passos de escritas diferentes em despachante generico.", "",
        "## Limites", "",
        "- Clones analisados: corpos AST de funcoes com pelo menos 45 tokens.",
        "- Analise lexical de macros nao identifica todos os casos possiveis.",
        "- A mudanca exige cargo fmt, Clippy, testes Rust e contratos no CI.",
        "- O codemod transforma apenas a serializacao JSON seguida de unwrap* em chamada de escrita direta.",
        "",
    ])
    return "\n".join(lines)


def main():
    args = argparse.ArgumentParser()
    mode = args.add_mutually_exclusive_group(required=True)
    for name in ("audit", "apply", "check"):
        mode.add_argument("--" + name, action="store_true")
    selected = args.parse_args()
    metrics, files, exact, shape = audit()
    changes, updates = [], []
    for path, source, locations in files:
        if not locations:
            continue
        new = source
        for a, b, replacement in reversed(locations):
            new = new[:a] + replacement + new[b:]
        if new != source:
            changes.append((str(path.relative_to(ROOT)), len(locations)))
            updates.append((path, new))
    if selected.check:
        if changes:
            raise SystemExit("Codemod ainda tem alteracoes: nao esta idempotente")
        print(json.dumps({"idempotente": True, "metricas": metrics}, ensure_ascii=False))
        return
    if selected.apply:
        if changes:
            path = SRC / "saida.rs"
            updates.append((path, helper(path.read_bytes())))
        for path, source in updates:
            path.write_bytes(source)
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        REPORT.write_text(report(metrics, exact, shape, changes), encoding="utf-8")
    print(json.dumps({"metricas": metrics, "alteracoes": changes,
                      "clones_exatos": [g[:5] for g in exact[:8]],
                      "clones_forma": [g[:5] for g in shape[:8]]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
