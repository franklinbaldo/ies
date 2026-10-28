---
type: Cassette
title: processo-escritas-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: processo-escritas-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 841b0341a18485fbb67cba42cb641ab6c4bb3ddc97e5fbd7ea146a6302045630
interactions: 40
schema_version: 1
complete: true
---

# processo-escritas-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `processo-escritas-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `841b0341a18485fbb67cba42cb641ab6c4bb3ddc97e5fbd7ea146a6302045630`. Commit de captura: `4d780be0a37ed067014f91324830c6e0ce821f87`.

**Observação:** 40 interações HTTP; métodos GET: 28, POST: 12.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `anotacao_registrar`: 4 requisição(ões)
- `procedimento_relacionar`: 4 requisição(ões)
- `procedimento_enviar`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `protocolo_RI1023`: 2 requisição(ões)
- `unidade_auto_completar_envio_processo`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_reabrir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
