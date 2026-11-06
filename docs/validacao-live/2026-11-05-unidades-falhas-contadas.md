---
type: SEIObservation
title: Contagem HTTP nas recusas da lupa de unidades
description: Issue 244; regressao sintética com contagem em falhas sem nova requisicao real ao SEI
---

# #244 — contagem das falhas Web (05/11/2026)

## Observação anterior ao RED

Os ensaios autenticados existentes mostram UnB recusando
`link_da_lupa_ambiguo` e GO recusando
`formulario_da_lupa_inesperado`, sem publicar a quantidade de
requisições feitas pela operação. Ver
[relatório precedente](2026-11-05-unidades-paginacao-integral.md),
[ensaio UnB 42143302555](https://github.com/franklinbaldo/ies/actions/runs/42143302555)
e [ensaio GO 42143615203](https://github.com/franklinbaldo/ies/actions/runs/42143615203).
Nenhuma captura comprova qual variação de HTML causou as recusas;
por isso não relaxamos validação de links ou formulários.

## Contrato e teste sintético

A coleta passa um contador mutável, inicializado em zero, pelos
GETs e POSTs da lupa. A tentativa é contada antes de chamar o
transporte. O mesmo contador é preservado em todas as recusas, sem
trocar o subcódigo de erro, expor cookie, URL assinada ou
`infra_hash`, ou repetir tentativa.

O shim inicial do commit `131578e` sempre reportava zero em
caso de falha. Dois testes novos, publicados em `4e83286`,
exigem (a) **5 requisições** na falha do primeiro POST
(4 GET + 1 POST) e (b) **278 requisições** na falha do último
POST da coleta sintética de 274 páginas (4 GET + 274 POST).
Os testes não usam rede. Teste da composição JSON verifica
`requisicoes` e ausência de campos privados.

**RED:** execução isolada não confirmada; o CI inicial desse head
foi cancelado por commits subsequentes. Não atribuir contagem
de falhas de teste ao RED sem log. **GREEN:** conferir no
CI final da PR #303; não antecipar PASS.

## Validação live desta fatia

| Etapa | Resultado | Requisições |
| --- | --- | --- |
| UnB — nova coleta | NOT_RUN | 0 |
| GO — nova coleta | NOT_RUN | 0 |
| Pesquisa offline pós-coleta | NOT_RUN | 0 |
| Escrita de domínio no SEI | NOT_RUN | 0 |

Nenhuma requisição ao SEI foi feita nesta fatia. Não repetir
as chamadas falhadas dos ensaios anteriores apenas para colher a
contagem. Não há novo resíduo de processo, documento ou unidade.
Replay GO já obrigatório na CI canônica; esta alteração não muda
cassette, protocolo ou conteúdo das páginas.

## Pendente na issue

Resolver as diferenças estruturais reais de UnB e GO com evidência,
obter a lista integral, pesquisar offline com zero HTTP e capturar
stdout/stderr real da CLI de sugestões. A contagem nas falhas
não é prova de sucesso da listagem nem conclui a issue.
