---
type: Cassette
title: processo-anexar-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-anexar-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 031b1681716a16d193eb4be258817e233bf47725c458cd0fbe2ef784883368fd
interactions: 51
schema_version: 1
complete: true
---

# processo-anexar-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-anexar-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `031b1681716a16d193eb4be258817e233bf47725c458cd0fbe2ef784883368fd`. Commit de captura: `70c2cd12d1cde1351832c09a862fbd05d62c43fd`.

**Observação:** 51 interações HTTP; métodos GET: 36, POST: 15.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_anexar`: 7 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `protocolo_RI1023`: 4 requisição(ões)
- `procedimento_anexacao_verificar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `documento_escolher_tipo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)
- `documento_visualizar`: 1 requisição(ões)
- `procedimento_concluir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
