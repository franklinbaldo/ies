---
type: Cassette
title: documento-mover-cancelar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: documento-mover-cancelar-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: a152c0306b45aa98b04be689f7a031aca1739692d91a2e41fe4291fbd70a2d08
interactions: 80
schema_version: 1
complete: true
---

# documento-mover-cancelar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `documento-mover-cancelar-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `a152c0306b45aa98b04be689f7a031aca1739692d91a2e41fe4291fbd70a2d08`. Commit de captura: `e4d5761f645bae3d5baddeafba1f6244f345eb31`.

**Observação:** 80 interações HTTP; métodos GET: 51, POST: 29.

## Operações HTTP registradas

- `procedimento_visualizar`: 18 requisição(ões)
- `procedimento_escolher_tipo`: 8 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `documento_escolher_tipo`: 6 requisição(ões)
- `documento_receber`: 5 requisição(ões)
- `documento_mover`: 5 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `documento_recebido_duplicado`: 3 requisição(ões)
- `protocolo_RI1023`: 3 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `documento_cancelar`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
