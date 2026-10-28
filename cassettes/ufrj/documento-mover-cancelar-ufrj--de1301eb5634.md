---
type: Cassette
title: documento-mover-cancelar-ufrj
installation: ufrj
source_zip: https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0350d4efa37710c9.zip
archive_member: documento-mover-cancelar-ufrj.json
archive_sha1: c95d0ad385617615974fad4c98137fcf3fa1b164
cassette_sha256_lf: de1301eb56343b0e3a66f47a906f20c0463f11d28624119289fc3940db133ee5
interactions: 70
schema_version: 1
complete: true
---

# documento-mover-cancelar-ufrj

**Instalação:** ufrj (`teste.sei.tic.ufrj.br`). **Origem:** [ZIP imutável do Internet Archive](https://archive.org/download/todos-sei-cassettes-2026/lotes/lote-0350d4efa37710c9.zip); membro `documento-mover-cancelar-ufrj.json`.

**Verificações:** SHA-1 do ZIP `c95d0ad385617615974fad4c98137fcf3fa1b164`; SHA-256 da cassette após CRLF→LF `de1301eb56343b0e3a66f47a906f20c0463f11d28624119289fc3940db133ee5`. Commit de captura: `5fe41adf9447f80c541480d1dae9153b75aea9c7`.

**Observação:** 70 interações HTTP; métodos GET: 46, POST: 24.

## Operações HTTP registradas

- `procedimento_visualizar`: 16 requisição(ões)
- `procedimento_escolher_tipo`: 8 requisição(ões)
- `procedimento_gerar`: 6 requisição(ões)
- `documento_escolher_tipo`: 6 requisição(ões)
- `documento_receber`: 5 requisição(ões)
- `procedimento_concluir`: 4 requisição(ões)
- `documento_recebido_duplicado`: 3 requisição(ões)
- `documento_imprimir_web`: 3 requisição(ões)
- `documento_cancelar`: 3 requisição(ões)
- `procedimento_trabalhar`: 2 requisição(ões)
- `documento_gerar`: 2 requisição(ões)
- `arvore_visualizar`: 2 requisição(ões)
- `documento_assinar`: 2 requisição(ões)
- `procedimento_enviar`: 2 requisição(ões)
- `procedimento_controlar`: 1 requisição(ões)
- `documento_upload_anexo`: 1 requisição(ões)
- `editor_montar`: 1 requisição(ões)
- `editor_salvar`: 1 requisição(ões)
- `unidade_auto_completar_envio_processo`: 1 requisição(ões)
- `procedimento_consultar_historico`: 1 requisição(ões)

## Alcance da evidência

A gravação comprova respostas observadas na instalação indicada. Não demonstra,
por si só, paridade com a CLI IES, corpo de requisições POST ou sucesso sem
conferência pós-escrita. Use os passos relevantes e o contrato antes de implementar.

Consulte [como o SEI funciona](../../docs/como-o-sei-funciona/README.md) e
[o procedimento de replay](../../docs/cassettes.md).
