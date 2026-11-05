---
type: ValidationReport
title: Auditoria da contagem HTTP nos fluxos de unidade — issue 235
description: Cobertura inspecionada e verificações ainda pendentes em 5 de novembro de 2026
---

# Issue #235 — auditoria e validação pendente

**Estado: NOT_RUN.** Este documento não representa implementação nem
validação concluída. A análise foi feita na árvore do branch
`ccr-712112d8-5zf61h`, porque a PR #233 ainda estava aberta.

## Observações do código

- `unidade atual` usa `relogin_cli::ler` e obtém a tela do Controle.
  Em sucesso sem relogin, o JSON não contém `requisicoes`.
- `listar-processos` usa `relogin_cli::ler`, GET inicial e o fluxo
  `visao_do_controle::listar_na_visao_detalhada`; a troca R→D e a
  paginação podem acrescentar POSTs. Não há `requisicoes` próprio na
  saída normal.
- `recebidos`/`gerados` usam `relogin_cli::ler` e
  `unidade_categoria::pagina_web::obter`. A saída normal também não
  publica `requisicoes`.
- `colunas` usa `relogin_cli::ler`, GET inicial e
  `visao_do_controle::ler_colunas`; sem relogin, não publica
  `requisicoes`.
- `colunas --definir` consulta a sessão e o Controle fora de
  `relogin_cli::ler`; seu JSON de sucesso não publica
  `requisicoes`. A variante com `--confirmar` pode salvar e conferir
  por leitura: ela **não** deve ser executada em treinamento sob esta
  autorização, que abrange apenas leituras.
- `relogin_cli::anotar_com_total` introduz `requisicoes` somente
  quando existe `NotaDoRelogin`. A contagem do login já é apurada pelo
  transporte do processo. Alterar a forma pública de saída exige decisão
  do proprietário.

## Gate e cobertura pendentes

| Verificação | Estado | Motivo |
| --- | --- | --- |
| RED dos novos testes | NOT_RUN | Nenhum teste novo foi executado; não há RED observado |
| Rust ≥ 1.85 / cargo | NOT_RUN | Executor local sem cargo; download externo indisponível por DNS |
| `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files` | NOT_RUN | Sem checkout local do repositório |
| CI da implementação | NOT_RUN | Nenhuma implementação foi apresentada |
| GET 200, 302 como dado, falha na segunda, POST recusado | NOT_RUN | Faltam testes injetados e execução |
| Treinamento, comandos de leitura | NOT_RUN | Nenhuma requisição foi enviada; sem risco de mutação |

## Próxima implementação necessária

Instrumentar as tentativas efetivamente enviadas por `EnvioDoControle`
e pelos envios iniciais dos comandos, antes da chamada HTTP, sem contar
parsing nem índice local; conferir as somas de relogin e os números
anteriores dos caminhos de sucesso. Usar `testes_http::atender`, observar
RED antes da implementação, executar o gate e publicar um novo registro
de validação live somente depois das leituras efetivas, obedecendo ao
intervalo mínimo entre comandos. Não introduzir campo público de
`requisicoes` nos fluxos que ainda não o possuem.

Nenhum literal de subcódigo foi alterado; `src/transporte/**`,
arquivos do cache de documentos e as frentes paralelas permaneceram intactos.
