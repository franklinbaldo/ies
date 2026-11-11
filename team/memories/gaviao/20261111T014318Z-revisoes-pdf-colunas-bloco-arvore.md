---
type: AgentMemory
title: "Revisões #734, #736, #737 e #724: prova sintética e gates do SHA"
description: "Quatro pareceres; #734 LGTM 7/7 no head a389fff; #736/#737/#724 CI rápido verde nos novos heads, oficial em execução. Atenção a 302, Rustfmt e provas independentes."
agent: gaviao
timestamp: "2026-11-11T01:43:18Z"
tags: [revisao, bdd, pdf, colunas, blocos, sqlite, cassettes, ci, cafezinho]
---

# Retomar na próxima rodada

- **#734 head `a389fff`:** [parecer](https://github.com/franklinbaldo/ies/pull/734#issuecomment-6784500500). CI oficial [#42938647577](https://github.com/franklinbaldo/ies/actions/runs/42938647577) **7/7**; a CLI real recusa PDF de outro NUP antes/depois de anexação, com e sem `--ocultar-inacessiveis`, sem conteúdo cruzado. Duas memórias OKF avançaram `main` após o head. Integrador deve conferir sincronização e SHA.
- **#736 head `6e63a26`:** [parecer](https://github.com/franklinbaldo/ies/pull/736#issuecomment-6784501348), CI rápido PASS e [oficial #42940746436](https://github.com/franklinbaldo/ies/actions/runs/42940746436) em execução. O head `753f67c` já passou 7/7; avanço da branch incorporou somente duas memórias OKF. Reavaliar CI no **novo SHA**, sem atribuir PASS antecipado.
- **#737 head `4b2f49a`:** [parecer](https://github.com/franklinbaldo/ies/pull/737#issuecomment-6784502205), CI rápido PASS após resolver Rustfmt (falha comprovada no head anterior); [oficial #42940700484](https://github.com/franklinbaldo/ies/actions/runs/42940700484) em execução. Prévia de bloco interno, segunda CLI no mesmo SQLite, colisão entre tipos e página parcial; reunião não testada positivamente. Card PEN1 com 41 interações, não paridade.
- **#724 head `631916c`:** [parecer](https://github.com/franklinbaldo/ies/pull/724#issuecomment-6784503075), CI rápido PASS, [oficial #42940593071](https://github.com/franklinbaldo/ies/actions/runs/42940593071) em execução. Regressão de pasta incompleta exige causa tipada sem falso documento ausente. Head anterior tinha `too_many_lines` e desvio 302; não deduzir GREEN pleno antes do gate.
- **Evidências:** sintético PASS apenas nas fatias realmente executadas pelo CI; cards PEN1, UnB e GO foram consultados e seus hashes verificados em runs anteriores, mas **confronto semântico bilateral dessas transições NOT_RUN**; treinamento/live **NOT_RUN**. Nenhum GET/POST ao SEI externo, merge ou implementação nesta rodada.
- **Cafezinho #388:** li a [conversa da 🦊 sobre a identidade de um bloco](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6784282623) e da [🐢 sobre recusa após anexação](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6784291240). Distinção útil: posição, tipo e parentesco não são identidade; analogia de revisão, não validação live.

Próximo ciclo: verificar CI completo no SHA final, diferenças com `main` e novos heads antes de duplicar parecer. Manter prioridade geral de ablação #716 sem transformar evidência sintética em prova externa.
