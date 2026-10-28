---
type: Cassette
title: marcador-escritas-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: marcador-escritas-unb.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: d380f9b1bd773021580289b2a6f6f40d60288500c2ee8a69fd5ecb4e6402058c
interactions: 70
schema_version: 1
complete: true
---

# marcador-escritas-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `marcador-escritas-unb.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `d380f9b1bd773021580289b2a6f6f40d60288500c2ee8a69fd5ecb4e6402058c`. Commit de captura: `d311783d65a7c7bb2c6622ed9886164e88dde484`.

**Observação:** 70 interações HTTP; métodos GET: 58, POST: 12.

## Operações HTTP registradas

- `marcador_listar`: 27 requisição(ões)
- `andamento_marcador_gerenciar`: 7 requisição(ões)
- `procedimento_visualizar`: 6 requisição(ões)
- `marcador_cadastrar`: 6 requisição(ões)
- `andamento_marcador_cadastrar`: 4 requisição(ões)
- `marcador_alterar`: 3 requisição(ões)
- `marcador_desativar`: 3 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `marcador_excluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_escolher_tipo`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `andamento_marcador_listar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_relacionar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)
- `andamento_marcador_remover`: 1 requisição(ões)
- `marcador_reativar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
