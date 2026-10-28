---
type: Cassette
title: grupo-envio-ciclo-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-44b94b71f1d40c2e.zip
archive_member: grupo-envio-ciclo-go.json
archive_sha1: 44b94b71f1d40c2eaa85a0517000e7142fdd2e6d
cassette_sha256_lf: 1cd5bea7daf5f9639abd239c3909de5f826deb8fc81b8dae8ef76b4b23d02eba
interactions: 35
schema_version: 1
complete: true
---

# grupo-envio-ciclo-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-44b94b71f1d40c2e.zip); membro `grupo-envio-ciclo-go.json`.

**Verificações:** SHA-1 do ZIP `44b94b71f1d40c2eaa85a0517000e7142fdd2e6d`; SHA-256 da cassette após CRLF→LF `1cd5bea7daf5f9639abd239c3909de5f826deb8fc81b8dae8ef76b4b23d02eba`. Commit de captura: `e7202f950332fdd6a5046b30fd5b12cfd2d9e16e`.

**Observação:** 35 interações HTTP; métodos GET: 27, POST: 8.

## Operações HTTP registradas

- `grupo_unidade_listar`: 16 requisição(ões)
- `grupo_unidade_alterar`: 6 requisição(ões)
- `unidade_auto_completar_envio_processo`: 4 requisição(ões)
- `grupo_unidade_consultar`: 4 requisição(ões)
- `grupo_unidade_cadastrar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `grupo_unidade_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
