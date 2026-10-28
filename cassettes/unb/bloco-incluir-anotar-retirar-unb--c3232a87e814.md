---
type: Cassette
title: bloco-incluir-anotar-retirar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-incluir-anotar-retirar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: c3232a87e81418e7a2a25975c48e2ca989d84bfd34710568fa54d620518b9190
interactions: 152
schema_version: 1
complete: true
---

# bloco-incluir-anotar-retirar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-incluir-anotar-retirar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `c3232a87e81418e7a2a25975c48e2ca989d84bfd34710568fa54d620518b9190`. Commit de captura: `3ad608ee8975fe7a01edb6c73fb87014adb1f6d5`.

**Observação:** 152 interações HTTP; métodos GET: 143, POST: 9.

## Operações HTTP registradas

- `procedimento_controlar`: 64 requisição(ões)
- `bloco_assinatura_listar`: 39 requisição(ões)
- `rel_bloco_protocolo_listar`: 22 requisição(ões)
- `rel_bloco_protocolo_alterar`: 6 requisição(ões)
- `protocolo_pesquisa_rapida`: 5 requisição(ões)
- `procedimento_trabalhar`: 5 requisição(ões)
- `procedimento_visualizar`: 5 requisição(ões)
- `bloco_escolher`: 5 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
