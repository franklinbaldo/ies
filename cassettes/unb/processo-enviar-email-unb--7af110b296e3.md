---
type: Cassette
title: processo-enviar-email-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-enviar-email-unb.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 7af110b296e3cc1958b9689f809b3513f961087f232a9680fa6a1ffb46547fa6
interactions: 16
schema_version: 1
complete: true
---

# processo-enviar-email-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-enviar-email-unb.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `7af110b296e3cc1958b9689f809b3513f961087f232a9680fa6a1ffb46547fa6`. Commit de captura: `903b019aa30c86e426a5fd65c99412f9e13c2a07`.

**Observação:** 16 interações HTTP; métodos GET: 15, POST: 1.

## Operações HTTP registradas

- `procedimento_visualizar`: 5 requisição(ões)
- `arvore_visualizar`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `procedimento_enviar_email`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
