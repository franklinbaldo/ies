---
type: AgentMemory
title: "Bloco incluir: idempotência não deve depender da ação de escrita oferecida"
description: "BDD-381-03 após a PR #701: leitura da presença de DOC ocorre depois da exigência do link e formulário de inclusão, potencial falha de no-op quando a ação some."
agent: polvo
timestamp: "2026-11-10T21:52:26Z"
tags: [blocos, documentos, bdd, idempotencia, anti-ies, estado-observado]
---

# Para retomar

- **Fato no head da [PR #701](https://github.com/franklinbaldo/ies/pull/701):** a função `prever_documento` exige link `bloco_escolher` e formulário com DOC antes de consultar o detalhe do bloco. A PR corrigiu o vínculo checkbox↔identidade; 🦅 aprovou o SHA `8cf6ba75` e CI oficial 7/7. PR ainda aberta na inspeção desta rodada.
- **Hipótese falsificável do BDD-381-03, não bug live comprovado:** se DOC já estiver em bloco e a barra não oferecer mais o link de inclusão, a saída pode ser recusa em vez de idempotência sem POST. Propor E2E CLI real↔anti-IES↔mesmo SQLite com DOC previamente presente e ação ausente, além de caso negativo DOC ausente com ação ausente. Registrei [contraprova na #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6780236771). Não alterei a branch da 🦊.
- **Próximo passo:** depois que 🐘/🦡 integrar #701, assumir BDD-381-03 em branch nova da `main`, inspecionar cards existentes antes de fixture e fazer RED/GREEN vertical. Provas desta nota: leitura estática; E2E novo NOT_RUN, confronto bilateral de cassettes NOT_RUN, live NOT_RUN. Sem SEI externo.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). As conversas recentes distinguem estado observado de ausência de observação; aqui o detalhe completo de bloco deve poder provar presença sem requerer um convite para escrever.
