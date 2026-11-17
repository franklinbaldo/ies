---
type: AgentMemory
title: "Tela de envio: doze recusas preservadas, supressão removida"
description: "PR #773 separou casos de recusa de suas asserções, corrigiu lint no CI e foi integrada com gates verdes; ZIP GO íntegro, paridade semântica não ensaiada."
agent: raposa
timestamp: "2026-11-11T10:25:02Z"
tags: [ux, ablacao, processo-enviar, rust, clippy, cassette-first, ci, cafezinho]
---

# Para retomar

**Fato:** a [PR #773](https://github.com/franklinbaldo/ies/pull/773) da [#721](https://github.com/franklinbaldo/ies/issues/721) foi integrada na `main` no merge `b2d40e9b`. Uma supressão `too_many_lines` foi retirada do teste da tela de envio; as 12 alterações sintéticas e as mesmas asserções permanecem, sem tocar produção. O primeiro CI revelou `needless_borrows_for_generic_args`; o commit `92bf46a6` corrigiu a chamada da closure sem novo `allow`.

**Provas:** [CI rápido #43002661401](https://github.com/franklinbaldo/ies/actions/runs/43002661401) PASS; [CI oficial #43002676466](https://github.com/franklinbaldo/ies/actions/runs/43002676466) **7/7 PASS**. O ZIP público [GO processo-escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/go/processo-escritas-go--841b0341a184.md) foi recuperado e conferido por SHA-1; membro JSON por SHA-256 LF, ambos PASS (40 interações). A validação atesta integridade da gravação, **não** confronto semântico IES e anti-IES, que permanece NOT_RUN. E2E novo do comando: NOT_RUN; treinamento/live: NOT_RUN.

**Cafezinho:** visitei e li a conversa [#388](https://github.com/franklinbaldo/ies/issues/388); a observação recente sobre releitura pós-POST reforça distinguir estado comprovado de diagnóstico sem certeza, sem criar uma obrigação para esta ablação puramente de teste.

**Próxima rodada:** escolher a próxima supressão comprovada da #721 sobre a `main` atual; consultar contrato e cassette antes, preservar as regressões Rust e não criar Gherkin nem subcódigos.
