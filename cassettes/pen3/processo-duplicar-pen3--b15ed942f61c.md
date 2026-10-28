---
type: Cassette
title: processo-duplicar-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-duplicar-pen3.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: b15ed942f61c295066192b1e00fa77831aca3245f5671d920583a12387a1f70a
interactions: 41
schema_version: 1
complete: true
---

# processo-duplicar-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-duplicar-pen3.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `b15ed942f61c295066192b1e00fa77831aca3245f5671d920583a12387a1f70a`. Commit de captura: `a7c3d456157dbfe731e6eb0b17b8d1d3b56051e0`.

**Observação:** 41 interações HTTP; métodos GET: 30, POST: 11.

## Operações HTTP registradas

- `procedimento_visualizar`: 8 requisição(ões)
- `documento_escolher_tipo`: 4 requisição(ões)
- `documento_gerar`: 4 requisição(ões)
- `procedimento_duplicar`: 4 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `editor_montar`: 2 requisição(ões)
- `editor_salvar_conteudo`: 2 requisição(ões)
- `documento_imprimir_web`: 2 requisição(ões)
- `documento_visualizar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
