---
type: Cassette
title: bloco-interno-processos-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip
archive_member: bloco-interno-processos-pen1.json
archive_sha1: 70841f418106b058c4f81c9b13a1322c51ce8cd0
cassette_sha256_lf: 69795bb274d66866ef28f3e2199f89fc5c31aa7f1715226f849e42118e911244
interactions: 41
schema_version: 1
complete: true
---

# bloco-interno-processos-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen1.zip); membro `bloco-interno-processos-pen1.json`.

**Verificações:** SHA-1 do ZIP `70841f418106b058c4f81c9b13a1322c51ce8cd0`; SHA-256 da cassette após CRLF→LF `69795bb274d66866ef28f3e2199f89fc5c31aa7f1715226f849e42118e911244`. Commit de captura: `53c88c729393a79adf2bded88c675cd2d6471486`.

**Observação:** 41 interações HTTP; métodos GET: 34, POST: 7.

## Operações HTTP registradas

- `bloco_interno_listar`: 13 requisição(ões)
- `rel_bloco_protocolo_listar`: 7 requisição(ões)
- `procedimento_visualizar`: 5 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `bloco_interno_cadastrar`: 2 requisição(ões)
- `bloco_selecionar_processo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `rel_bloco_protocolo_cadastrar`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
