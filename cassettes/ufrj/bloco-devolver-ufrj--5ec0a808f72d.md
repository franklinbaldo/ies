---
type: Cassette
title: bloco-devolver-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: bloco-devolver-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: 5ec0a808f72d4e37f5523701ddd7a625714d16c5cc58c596e8575c40ac2f36cc
interactions: 78
schema_version: 1
complete: true
---

# bloco-devolver-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `bloco-devolver-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `5ec0a808f72d4e37f5523701ddd7a625714d16c5cc58c596e8575c40ac2f36cc`. Commit de captura: `7a6483668fba2f0c4c4170dcc309816518f5c641`.

**Observação:** 78 interações HTTP; métodos GET: 54, POST: 24.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 25 requisição(ões)
- `procedimento_visualizar`: 7 requisição(ões)
- `procedimento_controlar`: 5 requisição(ões)
- `rel_bloco_protocolo_listar`: 4 requisição(ões)
- `infra_trocar_unidade`: 4 requisição(ões)
- `procedimento_trabalhar`: 3 requisição(ões)
- `bloco_assinatura_alterar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `bloco_assinatura_cadastrar`: 2 requisição(ões)
- `bloco_escolher`: 2 requisição(ões)
- `protocolo_pesquisa_rapida`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `unidade_auto_completar_outras`: 1 requisição(ões)
- `bloco_disponibilizar`: 1 requisição(ões)
- `bloco_retornar`: 1 requisição(ões)
- `rel_bloco_protocolo_excluir`: 1 requisição(ões)
- `bloco_excluir`: 1 requisição(ões)
- `documento_excluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
