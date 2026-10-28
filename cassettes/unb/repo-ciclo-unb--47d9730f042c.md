---
type: Cassette
title: repo-ciclo-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-eb4d07a9c441a2ef.zip
archive_member: repo-ciclo-unb.json
archive_sha1: eb4d07a9c441a2ef9c28c146aa0d59d9a78a0eda
cassette_sha256_lf: 47d9730f042c751e16d4d231ded3a7a6b5e845df264ab2d21e0d1b26a1ff55e4
interactions: 65
schema_version: 1
complete: true
---

# repo-ciclo-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-eb4d07a9c441a2ef.zip); membro `repo-ciclo-unb.json`.

**Verificações:** SHA-1 do ZIP `eb4d07a9c441a2ef9c28c146aa0d59d9a78a0eda`; SHA-256 da cassette após CRLF→LF `47d9730f042c751e16d4d231ded3a7a6b5e845df264ab2d21e0d1b26a1ff55e4`. Commit de captura: `639d39a7c0711887e46782108bc224e3a0fd502b`.

**Observação:** 65 interações HTTP; métodos GET: 33, POST: 32.

## Operações HTTP registradas

- `texto_padrao_interno_listar`: 48 requisição(ões)
- `infra_trocar_unidade`: 9 requisição(ões)
- `texto_padrao_interno_cadastrar`: 3 requisição(ões)
- `texto_padrao_interno_consultar`: 3 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `texto_padrao_interno_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
