---
type: Cassette
title: processo-arquivo-documentos-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-arquivo-documentos-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 4462ca5487d7086bbe06eeb615ae1cb598e01968559fefe57065f14c1873773e
interactions: 16
schema_version: 1
complete: true
---

# processo-arquivo-documentos-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-arquivo-documentos-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `4462ca5487d7086bbe06eeb615ae1cb598e01968559fefe57065f14c1873773e`. Commit de captura: `a0aa877745f26a37556e187cbc40abdf6348cdbe`.

**Observação:** 16 interações HTTP; métodos GET: 13, POST: 3.

## Operações HTTP registradas

- `procedimento_gerar_pdf`: 5 requisição(ões)
- `procedimento_visualizar`: 3 requisição(ões)
- `documento_selecionar`: 3 requisição(ões)
- `exibir_arquivo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
