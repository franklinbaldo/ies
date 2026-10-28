---
type: Cassette
title: processo-escritas-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-escritas-ufrj.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: d21a20db9b10a89bf02e860fd5fba4a846e27258a5a2e79c89f5e865e30a5cb8
interactions: 40
schema_version: 1
complete: true
---

# processo-escritas-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-escritas-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `d21a20db9b10a89bf02e860fd5fba4a846e27258a5a2e79c89f5e865e30a5cb8`. Commit de captura: `eb0bcbc0fd713b61ac0c531ba513740b992e4cf2`.

**Observação:** 40 interações HTTP; métodos GET: 28, POST: 12.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 4 requisição(ões)
- `procedimento_gerar`: 4 requisição(ões)
- `anotacao_registrar`: 4 requisição(ões)
- `procedimento_relacionar`: 4 requisição(ões)
- `procedimento_enviar`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `protocolo_RI1023`: 2 requisição(ões)
- `unidade_auto_completar_envio_processo`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `procedimento_reabrir`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
