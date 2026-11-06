---
type: SEIObservation
title: Validação offline e estado live de ciência documental em 06/11/2026
description: Testes sintéticos e pendências de treinamento da issue 271
---

# Ciência documental — #271, PR #317

| Etapa | Resultado | Requisições ao SEI |
| --- | --- | ---: |
| Observação: Todos @323eb01, UnB 4.0.9 | REFERÊNCIA | 0 |
| RED da superfície CLI, GitHub Actions [42193308344](https://github.com/franklinbaldo/ies/actions/runs/42193308344) | FAIL esperado: 3 falhas novas, 0 pass, 0 ignored | 0 |
| Suíte Rust, [CI 42197217607](https://github.com/franklinbaldo/ies/actions/runs/42197217607) | PASS nos testes da ciência; Clippy detectou documentação e estilo, corrigidos em commits posteriores | 0 |
| CI consolidado do head final | Conferir o run correspondente ao SHA atual da [PR #317](https://github.com/franklinbaldo/ies/pull/317) | 0 |
| Replay das cassettes de ciência | NOT_RUN | 0 |
| Prévia na UnB treinamento | NOT_RUN | 0 |
| Prévia em segunda instalação de treinamento | NOT_RUN | 0 |
| Um único `--confirmar` (treinamento) | NOT_RUN | 0 |
| Verificação por releitura após escrita live | NOT_RUN | 0 |

Nenhum acesso ao SEI ou escrita ao vivo ocorreu nesta fatia;
**zero resíduos novos**. O orçamento dos testes sintéticos é 3 GET
na prévia, 7 GET em confirmação e prova, 6 GET quando o aceite não
fornece atividade; **apenas um** dos GETs pode mutar o documento.

Não há comprovação da segunda ciência e do replay completo; permanecem
pendências da issue. Nunca executar em produção. A sequência segura de
treinamento é prévia ao vivo → uma confirmação → leitura de prova,
com registro do documento e do resíduo, caso seja possível completá-la.

O teste de regressão `template_inerte_nao_autoriza_get_mutador` comprova que
texto inerte em `<template>` não autoriza GET de escrita. O leitor agora
considera somente scripts inline ativos e ignora scripts de dados e fontes
externas sem prova observada. A tentativa de RED isolado desse novo teste
foi cancelada no [CI 42197133423](https://github.com/franklinbaldo/ies/actions/runs/42197133423)
antes de obter resultado; **RED novo: NOT_RUN**, não se confunde com o
RED inicial da superfície da CLI (três falhas já documentadas).
