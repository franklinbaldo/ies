"""Recusa `null` emitido pelo código de produção (#749).

A saída das CLIs não traz `null`: a leitura dá valor ou erro tipado, e o
campo sem valor por natureza é omitido. Procura `Value::Null` e a sintaxe
JSON `: null` em `src/`, fora do código de teste (arquivos de teste,
acervo, cassettes e o trecho a partir de `#[cfg(test)] mod`).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

PROIBIDO = re.compile(r"Value::Null|:\s*null\b|serialize_none")
ARQUIVO_DE_TESTE = re.compile(r"(^|/)(tests?(_[\w]+)?\.rs|tests?/)|acervo|(^|/)cassette(/|\.rs)")
MODULO_DE_TESTE = re.compile(r"^\s*#\[cfg\(test\)\]\s*\n\s*(pub(\([^)]*\))?\s+)?mod\s", re.M)


def producao(texto: str) -> str:
    """O texto até o primeiro módulo de teste, sem comentários de linha."""
    corte = MODULO_DE_TESTE.search(texto)
    if corte:
        texto = texto[: corte.start()]
    return re.sub(r"//[^\n]*", "", texto)


STRUCT_SERIALIZADA = re.compile(
    r"#\[derive\([^)]*\bSerialize\b[^)]*\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+[^{;]*\{(?P<corpo>.*?)\n\}",
    re.S,
)
CAMPO_OPCIONAL = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(\w+):\s*Option<", re.M)


def opcionais_sem_omissao(texto: str) -> list[str]:
    """Campos `Option` de structs `Serialize` que viram `null` sem `skip_serializing_if`."""
    achados = []
    for struct in STRUCT_SERIALIZADA.finditer(texto):
        corpo = struct.group("corpo")
        for campo in CAMPO_OPCIONAL.finditer(corpo):
            antes = corpo[: campo.start()].rstrip().splitlines()[-2:]
            if not any("skip_serializing_if" in linha for linha in antes):
                achados.append(campo.group(1))
    return achados


def violacoes(raiz: Path) -> list[str]:
    achados = []
    for arquivo in sorted(raiz.rglob("*.rs")):
        relativo = arquivo.relative_to(raiz).as_posix()
        if ARQUIVO_DE_TESTE.search(relativo):
            continue
        texto = producao(arquivo.read_text())
        for numero, linha in enumerate(texto.splitlines(), 1):
            if PROIBIDO.search(linha):
                achados.append(f"src/{relativo}:{numero}: {linha.strip()}")
        for campo in opcionais_sem_omissao(texto):
            achados.append(f"src/{relativo}: campo Option sem skip_serializing_if: {campo}")
    return achados


def main() -> int:
    achados = violacoes(Path("src"))
    for achado in achados:
        print(achado)
    return 1 if achados else 0


if __name__ == "__main__":
    sys.exit(main())
