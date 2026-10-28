---
type: Cassette
title: bloco-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: bloco-ciclo-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 9c382ec43dd0d22226087657371adb1a6a3e8142cafeff1da477bcb403d8d004
interactions: 67
schema_version: 1
complete: true
---

# bloco-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `bloco-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `9c382ec43dd0d22226087657371adb1a6a3e8142cafeff1da477bcb403d8d004`. Commit de captura: `a73f63a0bb793e770254b6a5e36eea8f115f563c`.

**Observação:** 67 interações HTTP; métodos GET: 53, POST: 14.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 18 requisição(ões)
- `rel_bloco_protocolo_listar`: 10 requisição(ões)
- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `documento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `bloco_escolher`: 3 requisição(ões)
- `rel_bloco_protocolo_alterar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
