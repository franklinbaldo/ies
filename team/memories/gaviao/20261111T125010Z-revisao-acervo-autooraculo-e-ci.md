---
type: AgentMemory
title: "Paridade de anexação: evitar auto-oráculo; replays em andamento"
description: "Quatro pareceres por SHA: #785 causa de filesystem 7/7, #781 replay tornou-se assertivo e foi promovida Ready, #777 sem comparação externa independente, #782 colide no arquivo de replay; acompanhar novos heads."
agent: gaviao
timestamp: "2026-11-11T12:50:10Z"
tags: [revisao, cassette-first, antipatterns, replay, ci, cafezinho]
---

# Retomar no próximo ciclo

- [#785 head `9edd559`](https://github.com/franklinbaldo/ies/pull/785#issuecomment-6796231298): causa `NotADirectory` preservada, [CI completo #43020611076](https://github.com/franklinbaldo/ies/actions/runs/43020611076) 7/7 com replays reais; falha local sem cassette Web aplicável, branch estava um commit atrás da main.
- [#781 head `4e060a4`](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6796231855): novo assert transforma relato de troca em replay que falha diante de leitura recusada; CI rápido verde, promovi Draft→Ready, [CI oficial #43023498443](https://github.com/franklinbaldo/ies/actions/runs/43023498443) iniciou. Branch estava 27 commits atrás: reconciliar antes do aceite final.
- [#777 head `498d255`](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6796232465): teste novo semeia SQLite com IDs extraídos **pelo próprio parser do IES** do HTML original e compara o parser com sua projeção. Isso é auto-oráculo: se IES omitir ID, anti-IES pode omitir igual sem falhar. Exigir expectativa independente da cassette, além de CI completo; o 503 continua sintético e não capturado.
- [#782 head `abae08f`](https://github.com/franklinbaldo/ies/pull/782#issuecomment-6796270017): novo acervo de anexação cobre leitores originais, mas não anti-IES independente, e conflita pelo mesmo `src/processo_anexar/acervo.rs` com #777. 65 arquivos, `behind=31`, conflito e nenhum CI oficial encontrado nesse SHA; revisão de integração ainda pendente.
- [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6795636384): 🦊 descreveu bem a falha lógica do relatório de replay que podia ficar verde apesar de `Recusado`. Isso reforça a contraprova #781, mas execução do novo assert ainda precisa de log.

Sem alterações em branches alheias, merge, credenciais, produção, retry, Gherkin ou live. Revisar apenas heads novos na volta e verificar execução do replay nominal por SHA.
