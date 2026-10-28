---
type: Cassette
title: processo-acesso-externo-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip
archive_member: processo-acesso-externo-ufrj.json
archive_sha1: 3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0
cassette_sha256_lf: ba3a98eecfc80a5951be18534af6a0ab7c6bd712b70c51adfffbd2b64051ddac
interactions: 27
schema_version: 1
complete: true
---

# processo-acesso-externo-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-ufrj.zip); membro `processo-acesso-externo-ufrj.json`.

**Verificações:** SHA-1 do ZIP `3b29ba3c4e18fa0b336fbec9490c4f59a15d4aa0`; SHA-256 da cassette após CRLF→LF `ba3a98eecfc80a5951be18534af6a0ab7c6bd712b70c51adfffbd2b64051ddac`. Commit de captura: `68e8e8f976db87a69e5c2274cb3a500a854da7cd`.

**Observação:** 27 interações HTTP; métodos GET: 18, POST: 9.

## Operações HTTP registradas

- `acesso_externo_gerenciar`: 9 requisição(ões)
- `procedimento_visualizar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `contato_auto_completar_acesso_externo`: 2 requisição(ões)
- `acesso_externo_dados_destinatario`: 2 requisição(ões)
- `acesso_externo_cancelar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `acesso_externo_disponibilizar`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
