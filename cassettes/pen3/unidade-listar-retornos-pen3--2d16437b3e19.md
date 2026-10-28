---
type: Cassette
title: unidade-listar-retornos-pen3
installation: pen3
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-15a748cdb7c0fdbd.zip
archive_member: unidade-listar-retornos-pen3.json
archive_sha1: 15a748cdb7c0fdbd907c8b3b922fa0194ae8887c
cassette_sha256_lf: 2d16437b3e193593994bf708fa71fd43bdf16864de3a528dc27213840784b781
interactions: 28
schema_version: 1
complete: true
---

# unidade-listar-retornos-pen3

**Instalação:** pen3 (`sei.orgao3.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-15a748cdb7c0fdbd.zip); membro `unidade-listar-retornos-pen3.json`.

**Verificações:** SHA-1 do ZIP `15a748cdb7c0fdbd907c8b3b922fa0194ae8887c`; SHA-256 da cassette após CRLF→LF `2d16437b3e193593994bf708fa71fd43bdf16864de3a528dc27213840784b781`. Commit de captura: `12f2a8a78810be744776574f116733e3828d08ad`.

**Observação:** 28 interações HTTP; métodos GET: 22, POST: 6.

## Operações HTTP registradas

- `retorno_programado_listar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_visualizar`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
