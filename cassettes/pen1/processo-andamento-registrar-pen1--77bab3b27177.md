---
type: Cassette
title: processo-andamento-registrar-pen1
installation: pen1
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip
archive_member: processo-andamento-registrar-pen1.json
archive_sha1: 0306b45276ae0084b78e781051f7ab5fc07e2693
cassette_sha256_lf: 77bab3b2717746d0e4cc2da21732c687fa8680548023628c3ed9fe4ec5969478
interactions: 23
schema_version: 1
complete: true
---

# processo-andamento-registrar-pen1

**Instalação:** pen1 (`sei.orgao1.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0306b45276ae0084.zip); membro `processo-andamento-registrar-pen1.json`.

**Verificações:** SHA-1 do ZIP `0306b45276ae0084b78e781051f7ab5fc07e2693`; SHA-256 da cassette após CRLF→LF `77bab3b2717746d0e4cc2da21732c687fa8680548023628c3ed9fe4ec5969478`. Commit de captura: `a5ca56e552484f93cebbdc04cb48260335b92ae1`.

**Observação:** 23 interações HTTP; métodos GET: 20, POST: 3.

## Operações HTTP registradas

- `procedimento_visualizar`: 6 requisição(ões)
- `procedimento_atualizar_andamento`: 5 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_consultar_historico`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
