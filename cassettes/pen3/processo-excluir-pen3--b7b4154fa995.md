---
type: Cassette
title: processo-excluir-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip
archive_member: processo-excluir-pen3.json
archive_sha1: b81b84fca29299f2096fb128593ba52f2ac9a7ab
cassette_sha256_lf: b7b4154fa9953d437fc3b45cc0957d32b7b1cf6deae040b0e1f94e93a1bba86a
interactions: 35
schema_version: 1
complete: true
---

# processo-excluir-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip); membro `processo-excluir-pen3.json`.

**Verificações:** SHA-1 do ZIP `b81b84fca29299f2096fb128593ba52f2ac9a7ab`; SHA-256 da cassette após CRLF→LF `b7b4154fa9953d437fc3b45cc0957d32b7b1cf6deae040b0e1f94e93a1bba86a`. Commit de captura: `4762810bcf765bac39883532347d4d656d9dcc67`.

**Observação:** 35 interações HTTP; métodos GET: 25, POST: 10.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_excluir`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
