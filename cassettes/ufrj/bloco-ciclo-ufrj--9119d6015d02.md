---
type: Cassette
title: bloco-ciclo-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: bloco-ciclo-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 9119d6015d022a7a183074db3b484d3120a0be2bf51f6f21773f30c2dd4fd39b
interactions: 71
schema_version: 1
complete: true
---

# bloco-ciclo-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `bloco-ciclo-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `9119d6015d022a7a183074db3b484d3120a0be2bf51f6f21773f30c2dd4fd39b`. Commit de captura: `c385cf1c903343d8b21bdeed213eb99f37eefaf8`.

**Observação:** 71 interações HTTP; métodos GET: 56, POST: 15.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 18 requisição(ões)
- `rel_bloco_protocolo_listar`: 10 requisição(ões)
- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `documento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 3 requisição(ões)
- `documento_gerar`: 3 requisição(ões)
- `bloco_assinatura_cadastrar`: 3 requisição(ões)
- `bloco_escolher`: 3 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `rel_bloco_protocolo_alterar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
