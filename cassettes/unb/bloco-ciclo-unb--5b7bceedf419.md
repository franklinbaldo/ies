---
type: Cassette
title: bloco-ciclo-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: bloco-ciclo-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 5b7bceedf419147715ccfc65944daa911bf25b5c9503cb59ecca27cd119e09df
interactions: 67
schema_version: 1
complete: true
---

# bloco-ciclo-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `bloco-ciclo-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `5b7bceedf419147715ccfc65944daa911bf25b5c9503cb59ecca27cd119e09df`. Commit de captura: `caadf245a1cb99a573a74ff71deab5c104905faa`.

**Observação:** 67 interações HTTP; métodos GET: 57, POST: 10.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 18 requisição(ões)
- `rel_bloco_protocolo_listar`: 10 requisição(ões)
- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `bloco_escolher`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `rel_bloco_protocolo_alterar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
