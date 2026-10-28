---
type: Cassette
title: marcador-escritas-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
archive_member: marcador-escritas-go.json
archive_sha1: 42a08cd3714bb863d136ed98e517221510a31982
cassette_sha256_lf: 7c45b39d550f52c45653d5470384c2318c6ccb833cc2ef5e503fcb21c786c393
interactions: 90
schema_version: 1
complete: true
---

# marcador-escritas-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip); membro `marcador-escritas-go.json`.

**Verificações:** SHA-1 do ZIP `42a08cd3714bb863d136ed98e517221510a31982`; SHA-256 da cassette após CRLF→LF `7c45b39d550f52c45653d5470384c2318c6ccb833cc2ef5e503fcb21c786c393`. Commit de captura: `70e2cf6d9985d42cf8bdeec3598011cb6b0fd368`.

**Observação:** 90 interações HTTP; métodos GET: 65, POST: 25.

## Operações HTTP registradas

- `marcador_listar`: 45 requisição(ões)
- `andamento_marcador_gerenciar`: 7 requisição(ões)
- `procedimento_visualizar`: 6 requisição(ões)
- `marcador_cadastrar`: 6 requisição(ões)
- `andamento_marcador_cadastrar`: 4 requisição(ões)
- `marcador_alterar`: 3 requisição(ões)
- `marcador_desativar`: 3 requisição(ões)
- `procedimento_escolher_tipo`: 2 requisição(ões)
- `procedimento_gerar`: 2 requisição(ões)
- `marcador_excluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_trabalhar`: 1 requisição(ões)
- `andamento_marcador_listar`: 1 requisição(ões)
- `procedimento_alterar`: 1 requisição(ões)
- `anotacao_registrar`: 1 requisição(ões)
- `procedimento_atribuicao_cadastrar`: 1 requisição(ões)
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
