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

## Estado

- **Replay offline:** PASS, 14/14 paginas mensais em sete instalacoes,
  com hashes conferidos. Ver
  [relatorio do replay](../validacao-replay/2026-11-05-unidade-retornos.md).
- **Testes sinteticos:** orcamento previsto de 3 GETs no mes exibido,
  mais um por transicao com URL publicada; nao confundir essa contagem
  prevista com os GETs realmente observados nos runs de falha.
- **Validacao ao vivo:** **FAIL**, nao homologada, ate demonstrar
  leitura integral da tela de calendario em treinamento com
  origem e estrutura verificadas, sem retry.
- **Residuo SEI:** nenhum registro criado ou alterado.
- **Sandbox:** sem `rustc` e DNS para GitHub; compilacao Rust e suite
  foram executadas no GitHub Actions, e nao localmente.
