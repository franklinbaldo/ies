---
type: Cassette
title: documento-criar-corpo-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: documento-criar-corpo-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 8d351fe9cbfd7e0ed944181bec1c153a152d047b9803811ce0b939b033abc591
interactions: 32
schema_version: 1
complete: true
---

# documento-criar-corpo-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `documento-criar-corpo-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `8d351fe9cbfd7e0ed944181bec1c153a152d047b9803811ce0b939b033abc591`. Commit de captura: `6449c0b6704fd38aa1e7fc87e6889d688213efbd`.

**Observação:** 32 interações HTTP; métodos GET: 29, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_verificar_assinatura`: 2 requisição(ões)
- `editor_montar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
