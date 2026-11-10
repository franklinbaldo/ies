---
type: SEIObservation
title: Auditoria da contagem HTTP nos fluxos de unidade — issue 235
description: Cobertura inspecionada e verificações ainda pendentes em 5 de novembro de 2026
---

# Issue #235 — auditoria e validação pendente

**Estado: CI PASS; live NOT_RUN.** A contagem das chamadas injetadas foi
implementada, mas a validação ao vivo não foi executada. A análise foi feita na árvore do branch
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
  `requisicoes`. A execução com `--confirmar` pode salvar e conferir
  por leitura: ela **não** deve ser executada em treinamento sob esta
  autorização, que abrange apenas leituras.
- `relogin_cli::anotar_com_total` introduz `requisicoes` somente
  quando existe `NotaDoRelogin`. A contagem do login já é apurada pelo
  transporte do processo. Alterar a forma pública de saída exige decisão
  do proprietário.

## Gate e cobertura pendentes

| Verificação | Estado | Motivo |
| --- | --- | --- |
| RED dos novos testes | NOT_RUN | Os testes foram introduzidos após a implementação; RED anterior não foi observado |
| Rust ≥ 1.85 / cargo | PASS no CI; NOT_RUN local | GitHub Actions instalou e executou Rust; sandbox não dispõe de cargo e não resolve o servidor de download |
| `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files` | PASS no CI; NOT_RUN local | Gate oficial executado com sucesso no workflow 42049781166, commit `c6643bb8` |
| CI da implementação | PASS | Rustfmt, Clippy, suite Rust, OKF e demais gates do workflow 42049781166 aprovados |
| GET 200, 302 como dado, falha na segunda, POST recusado | PASS | Quatro testes com `testes_http::atender` em `src/visao_do_controle/tests_contagem.rs`; suite Rust aprovada no CI |
| Treinamento, comandos de leitura | NOT_RUN | Nenhuma requisição foi enviada; sem risco de mutação |

## Implementação e limites

`EnvioContado` contabiliza GETs e POSTs injetados antes de chamar o
transporte, incluindo 302, recusa e falha de rede. O código de produção
usa o decorador em `listar-processos`, `recebidos`/`gerados` vindos da
visão D, `colunas` e `colunas --definir`. A colheita do índice não passa
pelo contador. GETs iniciais e POSTs diretos continuam contabilizados
pelo medidor global do transporte real usado por `relogin_cli`.

Os quatro novos testes com `testes_http::atender` conferem,
respectivamente, 1 GET, 1 GET sem seguir 302, 2 POSTs com falha no
segundo e 1 POST recusado. A execução anterior às correções mostrou
falha de Rustfmt e de Clippy, mas **não constitui RED pré-implementação**.
Os testes antigos de sucesso passaram sem alterar os orçamentos.

Não se acrescentou `requisicoes` às saídas normais que não o publicavam.
A decisão sobre ampliar esses JSONs segue com o proprietário.
A execução em treinamento permanece NOT_RUN: nenhum pedido foi enviado;
não há evidência live para marcar PASS.

Nenhum literal de subcódigo foi alterado; `src/transporte/**`,
arquivos do cache de documentos e as frentes paralelas permaneceram intactos.
