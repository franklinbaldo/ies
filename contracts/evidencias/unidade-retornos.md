---
type: OperationEvidence
title: unidade.retornos
operation: unidade.retornos
source: todos-323eb01
source_commit: 323eb013a1e7fac42c50336f838373a2cbe23e4b
writes: false
ies_implemented: true
upstream_steps: 3
unresolved_steps: 2
---

# Evidencia da leitura de retornos programados (Refs #253)

A [implementacao](../../src/unidade_retornos_web.rs) usa como oraculo
a [rotina Todos](https://github.com/franklinbaldo/todos/blob/323eb01/src/todos/retorno_programado.py)
e o [teste da cassette UnB](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_unidade_listar_retornos_010.py).
O fluxo observado e: menu Retorno Programado, calendario com
`data_atual`, link Ver todo o mes com `data_inicial/data_final`,
duas tabelas identificadas por `summary`, caption com contagem.
A implementacao Rust nao porta a arquitetura de sessao do Todos.

O teste de fonte reproduz os cenarios de mes com dados, mes vazio,
link ausente, mes adjacente e tabela insuficiente. O CI do primeiro
commit RED contou 3 testes novos falhando (de 1364 executados,
outros 1361 passaram), antes do leitor GREEN. O Rust nao estava
instalado no sandbox e o DNS nao resolveu GitHub: RED local NOT_RUN.

## Limites da evidencia

Os arquivos ZIP de cassette do Todos nao sao copiados para as fixtures
do IES nesta fatia. **Replay: NOT_RUN**. **Live GO/PEN3: NOT_RUN**,
zero requisicoes ao SEI. Falta verificar a pagina real em ambas
instalacoes e suportar paginacao segura por tabela independente.
A implementacao recusa lista parcial.
