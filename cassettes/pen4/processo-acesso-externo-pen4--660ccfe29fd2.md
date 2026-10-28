---
type: Cassette
title: processo-acesso-externo-pen4
installation: pen4
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-1e2f128fe845906a.zip
archive_member: processo-acesso-externo-pen4.json
archive_sha1: 1e2f128fe845906a364058f0d5eb22a0308b4095
cassette_sha256_lf: 660ccfe29fd203bf9fe6990d5c23527dc6555e514e551120fa44ce46538a8ff1
interactions: 27
schema_version: 1
complete: true
---

# processo-acesso-externo-pen4

**Instalação:** pen4 (`sei.orgao4.tramita.processoeletronico.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-1e2f128fe845906a.zip); membro `processo-acesso-externo-pen4.json`.

**Verificações:** SHA-1 do ZIP `1e2f128fe845906a364058f0d5eb22a0308b4095`; SHA-256 da cassette após CRLF→LF `660ccfe29fd203bf9fe6990d5c23527dc6555e514e551120fa44ce46538a8ff1`. Commit de captura: `26ef36a2ea9f0b9be210d10a8c7247832d60168c`.

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
