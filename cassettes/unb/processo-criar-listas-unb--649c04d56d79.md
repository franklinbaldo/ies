---
type: Cassette
title: processo-criar-listas-unb
installation: unb
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip
archive_member: processo-criar-listas-unb.json
archive_sha1: 21e0559015c56c403daf8933e5040504a7b83837
cassette_sha256_lf: 649c04d56d7901447bc96ac151246fc4be2cba12a005daa28425f5f906125aa4
interactions: 13
schema_version: 1
complete: true
---

# processo-criar-listas-unb

**Instalação:** unb (`sei.treinamento.unb.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip); membro `processo-criar-listas-unb.json`.

**Verificações:** SHA-1 do ZIP `21e0559015c56c403daf8933e5040504a7b83837`; SHA-256 da cassette após CRLF→LF `649c04d56d7901447bc96ac151246fc4be2cba12a005daa28425f5f906125aa4`. Commit de captura: `7e00d98c3533de8c80de740cb99fdd0c3768a2c1`.

**Observação:** 13 interações HTTP; métodos GET: 8, POST: 5.

## Operações HTTP registradas

- `procedimento_gerar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `assunto_auto_completar_RI1223`: 2 requisição(ões)
- `contato_auto_completar_contexto_RI1225`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `procedimento_visualizar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
