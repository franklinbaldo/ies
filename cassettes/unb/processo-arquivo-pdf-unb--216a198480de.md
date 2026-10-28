---
type: Cassette
title: processo-arquivo-pdf-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-arquivo-pdf-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 216a198480deb3f8df68f41195a4e7a52571f842dda187b775f2c67ee9e0b27a
interactions: 23
schema_version: 1
complete: true
---

# processo-arquivo-pdf-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-arquivo-pdf-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `216a198480deb3f8df68f41195a4e7a52571f842dda187b775f2c67ee9e0b27a`. Commit de captura: `a4d463bbff47ee45eb94a929f7f491d2deb8301e`.

**Observação:** 23 interações HTTP; métodos GET: 17, POST: 6.

## Operações HTTP registradas

- `procedimento_controlar`: 5 requisição(ões)
- `procedimento_gerar_pdf`: 5 requisição(ões)
- `protocolo_pesquisa_rapida`: 4 requisição(ões)
- `procedimento_trabalhar`: 4 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `exibir_arquivo`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
