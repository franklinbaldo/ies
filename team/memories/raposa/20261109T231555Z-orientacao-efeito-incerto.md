---
type: AgentMemory
title: "Efeito incerto: consultar não significa repetir o POST"
description: "PR #615 corrige uma instrução perigosa da CLI após saída 3, com RED de duas regressões E2E e confirmação pendente no CI final."
agent: raposa
timestamp: "2026-11-09T23:15:55Z"
tags: [ux, processo-atribuir, efeito-incerto, cassette-first, anti-ies, sem-retry]
---

# Retomar

**Fato:** [PR #615](https://github.com/franklinbaldo/ies/pull/615) modifica a mensagem existente de `efeito_incerto_na_atribuicao`: antes dizia consultar *antes de repetir*, agora manda **não repetir a confirmação e consultar o processo** para avaliar possível resíduo. Nenhum novo subcódigo, payload, POST ou escrita externa. O [RED #42718235979](https://github.com/franklinbaldo/ies/actions/runs/42718235979) mostrou **2 FAIL / 11 PASS** no E2E da CLI real contra anti-IES/SQLite (302 histórico e 403 após POST aplicado). O JSON usa o campo `erro`, não `mensagem`: teste e contrato ajustados. [CI GREEN #42718783870](https://github.com/franklinbaldo/ies/actions/runs/42718783870) ainda em execução ao criar esta memória.

**Limites:** card UnB de 24 interações consultado, confronto causal de POST com cassette **NOT_RUN** nesta fatia; IES↔anti-IES precisa de resultado do CI GREEN, treinamento **NOT_RUN**, sem repetir a atribuição incerta de 09/11. O possível resíduo anterior continua possível, não se tornou comprovado nem limpo.

**Cafezinho:** a conversa da [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6752736877) separa conferência de hash e compatibilidade funcional; analogamente, estado relido igual ao solicitado não substitui prova de autoria da mudança.

**CONTINUAR:** conferir GREEN no head atual, sincronizar a branch com a memória publicada na main por merge verdadeiro, revalidar os gates, pedir revisão 🦅 e handoff de integração 🐘/🦡.
