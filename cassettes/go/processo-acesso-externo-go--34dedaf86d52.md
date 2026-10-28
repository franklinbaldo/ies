---
type: Cassette
title: processo-acesso-externo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-acesso-externo-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 34dedaf86d5280681c20abe00a6aa1a627118c9fe15918b5c637c2869dfa9a8a
interactions: 27
schema_version: 1
complete: true
---

# processo-acesso-externo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-acesso-externo-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `34dedaf86d5280681c20abe00a6aa1a627118c9fe15918b5c637c2869dfa9a8a`. Commit de captura: `e31a23af611e6cfdd66fbb86678e4b7b2f5dbfb5`.

**Observação:** 27 interações HTTP; métodos GET: 18, POST: 9.

## Operações HTTP registradas

- `acesso_externo_gerenciar`: 9 requisição(ões)
- `procedimento_visualizar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `contato_auto_completar_acesso_externo`: 2 requisição(ões)
- `acesso_externo_dados_destinatario`: 2 requisição(ões)
- `acesso_externo_cancelar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `acesso_externo_disponibilizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
