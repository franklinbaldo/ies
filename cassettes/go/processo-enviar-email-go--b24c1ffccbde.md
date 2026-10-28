---
type: Cassette
title: processo-enviar-email-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-22044799039c3b65.zip
archive_member: processo-enviar-email-go.json
archive_sha1: 22044799039c3b65aba900b43f4cc826fd2725c2
cassette_sha256_lf: b24c1ffccbdeca8ecfc0727b34c015d0e11a963f74a025f0d5ed7e35bc38a6bd
interactions: 22
schema_version: 1
complete: true
---

# processo-enviar-email-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-22044799039c3b65.zip); membro `processo-enviar-email-go.json`.

**Verificações:** SHA-1 do ZIP `22044799039c3b65aba900b43f4cc826fd2725c2`; SHA-256 da cassette após CRLF→LF `b24c1ffccbdeca8ecfc0727b34c015d0e11a963f74a025f0d5ed7e35bc38a6bd`. Commit de captura: `a5f5f54557de56c847da55e816005e9cfff71df0`.

**Observação:** 22 interações HTTP; métodos GET: 18, POST: 4.

## Operações HTTP registradas

- `procedimento_visualizar`: 7 requisição(ões)
- `procedimento_enviar_email`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `procedimento_consultar_historico`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
