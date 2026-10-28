---
type: Cassette
title: processo-anexar-go
installation: go
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip
archive_member: processo-anexar-go.json
archive_sha1: c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78
cassette_sha256_lf: 7dec4ec7e6f4dda4b611c60604b951000407cff24b849ebeec1a77e94e721b23
interactions: 51
schema_version: 1
complete: true
---

# processo-anexar-go

**Instalação:** go (`seitreinamento.go.gov.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-c7a4af8d9d70ddc3.zip); membro `processo-anexar-go.json`.

**Verificações:** SHA-1 do ZIP `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78`; SHA-256 da cassette após CRLF→LF `7dec4ec7e6f4dda4b611c60604b951000407cff24b849ebeec1a77e94e721b23`. Commit de captura: `7eb78bd3728df8ef0812c1caa73163a2885a4900`.

**Observação:** 51 interações HTTP; métodos GET: 31, POST: 20.

## Operações HTTP registradas

- `procedimento_visualizar`: 9 requisição(ões)
- `procedimento_escolher_tipo`: 8 requisição(ões)
- `procedimento_anexar`: 7 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `protocolo_RI1023`: 4 requisição(ões)
- `procedimento_anexacao_verificar`: 4 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_escolher_tipo`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `procedimento_concluir`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `arvore_visualizar`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar_conteudo`: 1 requisição(ões)
- `documento_imprimir_web`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
