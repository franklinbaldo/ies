---
type: Cassette
title: bloco-devolver-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip
archive_member: bloco-devolver-go.json
archive_sha1: a00f2dd917ad3a32683c93cc02f0bf0942699e9d
cassette_sha256_lf: d0951370f2d87dfd52e1b6443bb1d8e346f7b9bac8728baeef6c99f93c838d25
interactions: 80
schema_version: 1
complete: true
---

# bloco-devolver-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-a00f2dd917ad3a32.zip); membro `bloco-devolver-go.json`.

**Verificações:** SHA-1 do ZIP `a00f2dd917ad3a32683c93cc02f0bf0942699e9d`; SHA-256 da cassette após CRLF→LF `d0951370f2d87dfd52e1b6443bb1d8e346f7b9bac8728baeef6c99f93c838d25`. Commit de captura: `ab4aaf673a2bb9c77e6e5c688afbc2eacf69e443`.

**Observação:** 80 interações HTTP; métodos GET: 56, POST: 24.

## Operações HTTP registradas

- `bloco_assinatura_listar`: 26 requisição(ões)
- `procedimento_visualizar`: 7 requisição(ões)
- `procedimento_controlar`: 5 requisição(ões)
- `rel_bloco_protocolo_listar`: 5 requisição(ões)
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
- `editor_salvar_conteudo`: 1 requisição(ões)
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
