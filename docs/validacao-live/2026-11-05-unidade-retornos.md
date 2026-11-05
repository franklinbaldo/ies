---
type: SEIObservation
title: Validacao live de retornos programados GO e PEN3 em treinamento
description: Evidencias por etapa e orcamento observado no ensaio de 5 de novembro de 2026
---

# Unidade retornos — verificacao live de 05/11/2026

A execucao [42096207509](https://github.com/franklinbaldo/ies/actions/runs/42096207509)
usou apenas o SEI de treinamento de GO, as credenciais publicas
documentadas e arquivos de sessao privados com permissoes `0600`.

| Instalacao | Etapa | Resultado | Requisicoes |
| --- | --- | --- | --- |
| GO (treinamento) | `ies --json login` | **PASS** — `login_realizado` | **6** |
| GO (treinamento) | `ies --json unidade retornos --mes 2026-10` | **FAIL/INCONCLUSIVO** — job encerrou em 1 antes do resumo do comando | **nao observadas** |
| PEN3 (treinamento) | `ies --json login` | **NOT_RUN** — a etapa GO interrompeu o job antes do loop PEN3 | **0** |
| PEN3 (treinamento) | `ies --json unidade retornos --mes 2026-10` | **NOT_RUN** | **0** |
| GO e PEN3 | `ies unidade retornos` sem `--mes` | **NOT_RUN** | **0** |

O login GO exibiu unidade ativa e seis requisicoes. Apos espera
de 65 s, o job terminou em erro sem publicar o resumo saneado de
`retornos`; portanto **nao** se pode afirmar que a consulta teve
zero GETs, nem que produziu os dados corretos. Nao repetir a consulta GO
sem nova justificativa: a operacao foi tentada uma vez.

A execucao subsequente
[42097152925](https://github.com/franklinbaldo/ies/actions/runs/42097152925)
foi **NOT_RUN para rede SEI**: o guard de commit registrou
`NOT_RUN: este commit nao e o gatilho de leitura ao vivo`.

## Limites e seguranca

- O replay de cassettes tem relatorio separado em
  `docs/validacao-replay/2026-11-05-unidade-retornos.md`.
- GO: orcamento de login observado = 6; orcamento do comando de retornos
  **nao verificavel** a partir do log disponivel. A meta sintetica
  continua sendo 3 GETs para o mes exibido, mais um por passo.
- Nenhuma escrita de processo ou documento foi solicitada; residuo
  de escrita no SEI: **nenhum**.
- O sandbox local nao possui Rust e nao resolve `github.com` por DNS;
  compilacao e gate foram executados pelo CI, nao localmente.
