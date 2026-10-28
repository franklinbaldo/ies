---
type: Cassette
title: documento-mover-cancelar-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip
archive_member: documento-mover-cancelar-pen3.json
archive_sha1: b81b84fca29299f2096fb128593ba52f2ac9a7ab
cassette_sha256_lf: 60eb468849ded836d14044a22f5a29f62d25d605a9c2aefbf4d03f5362dcc9d4
interactions: 81
schema_version: 1
complete: true
---

# documento-mover-cancelar-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen3.zip); membro `documento-mover-cancelar-pen3.json`.

**Verificações:** SHA-1 do ZIP `b81b84fca29299f2096fb128593ba52f2ac9a7ab`; SHA-256 da cassette após CRLF→LF `60eb468849ded836d14044a22f5a29f62d25d605a9c2aefbf4d03f5362dcc9d4`. Commit de captura: `cc91f44c1fff14bfae0bd3b39f20685d6c145f85`.

**Observação:** 81 interações HTTP; métodos GET: 52, POST: 29.

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
- `documento_visualizar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
