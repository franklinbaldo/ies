---
type: Cassette
title: processo-alterar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-alterar-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: f97c433b5e31bbd2c5e40ba9e59f778cea5a0fe26cc7b932434b1a821c65cc82
interactions: 48
schema_version: 1
complete: true
---

# processo-alterar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-alterar-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `f97c433b5e31bbd2c5e40ba9e59f778cea5a0fe26cc7b932434b1a821c65cc82`. Commit de captura: `280d9b26fcf9ea1286cf362e8ae633c75217b998`.

**Observação:** 48 interações HTTP; métodos GET: 35, POST: 13.

## Operações HTTP registradas

- `procedimento_alterar`: 20 requisição(ões)
- `procedimento_visualizar`: 13 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `hipotese_legal_select_nome_base_legal`: 2 requisição(ões)
- `assunto_auto_completar_RI1223`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
