---
type: Cassette
title: processo-acesso-externo-pen2
installation: pen2
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip
archive_member: processo-acesso-externo-pen2.json
archive_sha1: 28a245e07bf7d358a322981744dc5e79a835ba94
cassette_sha256_lf: 4cab0467d307a965101361b20f9c4293e889fbc8ab8e75a4e7cd9718a1fb45da
interactions: 27
schema_version: 1
complete: true
---

# processo-acesso-externo-pen2

**Instalação:** pen2 (`sei.orgao2.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-28a245e07bf7d358.zip); membro `processo-acesso-externo-pen2.json`.

**Verificações:** SHA-1 do ZIP `28a245e07bf7d358a322981744dc5e79a835ba94`; SHA-256 da cassette após CRLF→LF `4cab0467d307a965101361b20f9c4293e889fbc8ab8e75a4e7cd9718a1fb45da`. Commit de captura: `aa79267728d31eeea617702e43e386faab3d7277`.

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
