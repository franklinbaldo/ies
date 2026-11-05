---
type: SEIObservation
title: Validacao live de retornos programados GO e PEN3 em treinamento
description: Dois logins observados e falhas de leitura em GO e PEN3 sem repeticao de pedidos
---

# Unidade retornos — verificacao live de 05/11/2026

Os ensaios usaram exclusivamente instalacoes **de treinamento**,
contas publicas documentadas e arquivos privados de sessao `0600`.
Nao houve retry, escrita de processo ou documento, nem resíduo de escrita.

| Instalacao de treinamento | Etapa | Resultado | Requisicoes |
| --- | --- | --- | --- |
| GO | `ies --json login` | **PASS** — `login_realizado`, unidade exibida | **6** |
| GO | `ies --json unidade retornos --mes 2026-10` | **FAIL/INCONCLUSIVO** — o job encerrou em erro antes do resumo saneado | **nao observadas** |
| PEN3 | `ies --json login` | **PASS** — `login_realizado`, unidade `TESTE` | **6** |
| PEN3 | `ies --json unidade retornos --mes 2026-10` | **FAIL** — saida 1, `fonte_invalida/calendario_dos_retornos_desconhecido` | **nao publicadas** |
| GO e PEN3 | `ies unidade retornos` sem `--mes` | **NOT_RUN** | **0** |

## Evidencia GO

[Run 42096207509](https://github.com/franklinbaldo/ies/actions/runs/42096207509):
o login GO foi concluido com 6 requisicoes. Houve espera de
65 segundos antes do comando de retornos. O job encerrou em erro
antes de exibir o retorno da CLI: nao ha contagem confiavel de GETs
do comando, nem prova de lista completa. Nao se repetiu a leitura.

## Evidencia PEN3

[Run 42118598153](https://github.com/franklinbaldo/ies/actions/runs/42118598153):
o login foi concluido com 6 requisicoes e unidade `TESTE`.
Apos 65 segundos de espera, uma unica chamada de `retornos --mes`
retornou codigo 1 e subcodigo
`calendario_dos_retornos_desconhecido`. O campo `requisicoes`
nao foi publicado na falha; logo o numero de GETs do comando
**nao foi medido**. A falta de link mensal reconhecido e uma
hipotese diagnostica, ainda nao comprovada por captura do HTML.
Nao repetir a mesma requisicao sem nova evidencia.

O [run 42097152925](https://github.com/franklinbaldo/ies/actions/runs/42097152925)
teve guard `NOT_RUN` para rede. O workflow temporario de live foi
removido apos o ensaio unico PEN3, para evitar repeticoes automatizadas.

## Nova rodada de leitura: PASS em PEN3 e GO (05/11/2026)

[Workflow de leitura unica](https://github.com/franklinbaldo/ies/actions/runs/42122485600),
head `9d73e95`. Foram usadas somente as origens oficialmente
documentadas de **treinamento**, contas publicas e arquivos privados
`0600`, com separacao de pelo menos 65 segundos entre os comandos.
O pedido novo foi **2026-11**; as tentativas inconclusivas anteriores
eram de **2026-10**, sem repeticao automatica. O workflow tinha guard
de commit unico e recusa de rerun, e foi removido depois do ensaio.

| Instalacao | Comando | Resultado | Requisicoes observadas | Resultado |
| --- | --- | --- | ---: | --- |
| PEN3 | `ies --json login` | **PASS**, unidade `TESTE` | 6 | Sessao de treinamento |
| PEN3 | `ies --json unidade retornos --mes 2026-11` | **PASS**, saida 0 | 3 | 2 retornos; faixa 1–2 |
| GO | `ies --json login` | **PASS**, unidade `SEAD/CDE-09516` | 6 | Sessao de treinamento |
| GO | `ies --json unidade retornos --mes 2026-11` | **PASS**, saida 0 | 3 | 6 retornos; faixa 1–6 |

**Total observado:** 18 requisicoes (12 dos dois logins, 6 dos dois
comandos de leitura). Cada leitura usou exatamente o orcamento
de **3 GET**, sem GET de restauracao, retry, POST de criacao,
acao assinada vazada, ou escrita de processo/documento. Nenhum
residuo foi deixado no SEI.

A causa tratada na fatia foi observada
[no replay estrutural](https://github.com/franklinbaldo/ies/actions/runs/42120788751):
duas ancoras `Ver todo o mes` apontavam para a mesma URL mensal
nas sete instalacoes. A deduplicacao estrita preserva recusa de URLs
diferentes. A anterior falha PEN3 fica historicamente registrada;
a nova leitura em outro mes passou. O campo `requisicoes` desta
nova rodada foi publicado pela propria CLI, ao contrario das
falhas anteriores.

**Limites remanescentes:** o ensaio nao verificou `--mes` ausente
em calendario deslocado, mes anterior, ou navegacao em mais de um
salto. A nova fatia permite procurar `diaHoje` quando o calendario
esta deslocado, usando somente links publicados e o UTC como pista;
a verificacao autenticada especifica permanece **NOT_RUN (0 pedidos)**.
Os testes sinteticos mostram o custo de 4 GET para mes vizinho,
sem permitir publicar uma data da maquina como se fosse do SEI.
Nao se repetiram os ensaios anteriores nem houve novas escritas. O resultado nao prova paridade em
instalacoes alem de GO e PEN3 em 05/11.

## Estado

- **Replay offline:** PASS, 14/14 paginas mensais em sete instalacoes,
  com hashes conferidos. Ver
  [relatorio do replay](../validacao-replay/2026-11-05-unidade-retornos.md).
- **Testes sinteticos:** orcamento previsto de 3 GETs no mes exibido,
  mais um por transicao com URL publicada; nao confundir essa contagem
  prevista com os GETs realmente observados nos runs de falha.
- **Validacao ao vivo (nova rodada):** **PASS** em PEN3 e GO para 2026-11,
  com 3 GET por leitura e 6 requisicoes por login. As tentativas anteriores
  em 2026-10 permanecem documentadas como falhas.
- **Residuo SEI:** nenhum registro criado ou alterado.
- **Sandbox:** sem `rustc` e DNS para GitHub; compilacao Rust e suite
  foram executadas no GitHub Actions, e nao localmente.
