---
type: Cassette
title: bloco-disponibilizar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-disponibilizar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: e5e244620c3c9b017537e21cada6b13b5a764112a65f83c8346ca33638b6467d
interactions: 54
schema_version: 1
complete: true
---

# bloco-disponibilizar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-disponibilizar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `e5e244620c3c9b017537e21cada6b13b5a764112a65f83c8346ca33638b6467d`. Commit de captura: `599a52e96d09849f76c4487c903fe58cae47fa70`.

**Observação:** 54 interações HTTP; métodos GET: 42, POST: 12.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 27 requisição(ões)
- `rel_bloco_protocolo_listar`: 6 requisição(ões)
- `bloco_assinatura_alterar`: 4 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `bloco_escolher`: 3 requisição(ões)
- `unidade_auto_completar_outras`: 2 requisição(ões)
- `procedimento_visualizar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `protocolo_pesquisa_rapida`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `bloco_disponibilizar`: 1 requisição(ões)
- `bloco_cancelar_disponibilizacao`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
