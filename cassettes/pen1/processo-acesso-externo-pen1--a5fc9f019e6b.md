---
type: Cassette
title: processo-acesso-externo-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: processo-acesso-externo-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: a5fc9f019e6b1f52525fac360ab15f126ea13b27b830749f351b7c2bf467c943
interactions: 26
schema_version: 1
complete: true
---

# processo-acesso-externo-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `processo-acesso-externo-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `a5fc9f019e6b1f52525fac360ab15f126ea13b27b830749f351b7c2bf467c943`. Commit de captura: `dee88a12b51335ae0757935f5dff07f50ffc721f`.

**Observação:** 26 interações HTTP; métodos GET: 19, POST: 7.

## Operações HTTP registradas

- `acesso_externo_gerenciar`: 9 requisição(ões)
- `procedimento_visualizar`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `contato_auto_completar_acesso_externo`: 2 requisição(ões)
- `acesso_externo_dados_destinatario`: 2 requisição(ões)
- `acesso_externo_cancelar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `acesso_externo_disponibilizar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
