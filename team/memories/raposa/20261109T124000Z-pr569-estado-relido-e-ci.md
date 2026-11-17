---
type: AgentMemory
title: "Ensaio de atribuição: estado relido não certifica autoria"
description: "PR #569 está sincronizada (behind=0) e validada no head aabefcf3 pelo CI oficial 7/7; aguarda integração. Registro distingue ausência observada, não observação e efeito incerto."
agent: raposa
timestamp: "2026-11-09T12:40:00Z"
tags: [ux, atribuicao, bdd, anti-ies, memoria, live, ci]
---

# Retomar

**Fato:** na [PR #569](https://github.com/franklinbaldo/ies/pull/569), a ferramenta de ensaio mantém `atribuicao_relida` no checkpoint de efeito incerto sem alegar que o POST causou o estado. O [CI oficial do head aabefcf3](https://github.com/franklinbaldo/ies/actions/runs/42617827446) passou **7/7**, incluindo a regressão Python (23 testes) no job existente. Na reconciliação, branch 0 atrás de `main`; [handoff](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6740286812). A PR está aberta e pronta para 🦅/🐘/🦡; não mexer no branch após merge.

**Evidências:** sintético anterior PASS; 28 formulários GET confrontados; 14 respostas POST caracterizadas, sem prova causal; live NOT_RUN. Próximo trabalho real é ensaio controlado UnB treinamento autorizado, com prévia e POSTs únicos, releitura, remoção, resíduo e sem repetição quando incerto.

**Cafezinho:** a conversa sobre `None` versus ausência explícita lembra que um campo nulo no checkpoint não quer dizer "sem atribuição". É analogia de estados lógicos, não evidência nova do SEI.
