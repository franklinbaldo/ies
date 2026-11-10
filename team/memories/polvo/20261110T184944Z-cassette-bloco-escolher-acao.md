---
type: AgentMemory
title: "Bloco incluir: inspeção distingue POST documental do inventário agregado"
description: "Workflow de cassettes UnB/PEN1 confirmou integridade e identificou um único POST bloco_escolher na UnB; a prévia BDD-381-01 pertence à PR #694 do Elefante e a escrita 02 permanece sem replay."
agent: polvo
timestamp: "2026-11-10T18:49:44Z"
tags: [bloco-incluir, documentos, cassettes, github-actions, anti-ies, bdd]
---

# Retomar na próxima rodada

- **Fato novo:** [inspeção Actions #42878787801](https://github.com/franklinbaldo/ies/actions/runs/42878787801), SHA-1/SHA-256 PASS em UnB (152 interações) e PEN1 (41), isolou **4 GET + 1 POST `bloco_escolher`** no primeiro card; no PEN1, `bloco_selecionar_processo` tem 2 GET, nenhum POST, e `rel_bloco_protocolo_cadastrar` tem 1 POST. Veja [comentário na #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6776202053).
- **Limite:** hash e inventário de métodos **não** provam corpo, aceite causal, transição ou releitura. Confronto semântico bilateral, novo E2E CLI↔anti-IES↔SQLite e live ficaram NOT_RUN. Não houve contato com SEI, credenciais ou escrita externa.
- **Estado concorrente:** o 🐘 assumiu **BDD-381-01**, prévia documental, na [PR #694](https://github.com/franklinbaldo/ies/pull/694) ainda aberta. Não editei a branch alheia; após integrar a 01, a próxima fatia livre é a **BDD-381-02**, confirmação documental causal com simulação SQLite e exatamente um POST.
- A [PR técnica transitória #696](https://github.com/franklinbaldo/ies/pull/696) foi **fechada sem merge** após retirar `.github/cassettes-a-inspecionar.txt`, conforme guia. Visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) após a inspeção: a conversa atual sobre estado parcial, identidade literal e evidência de escrita continua pertinente.
