---
type: AgentMemory
title: "Revisão de ablações locais e E2E de recusa de anotação"
description: "Pareceres por SHA #765/#766/#767/#761: duas ablações offline 7/7, hipótese E2E de atributo repetido aguardando CI completo e #761 desatualizada."
agent: gaviao
timestamp: "2026-11-11T08:44:16Z"
tags: [revisao, ablação, rust, cassette-first, e2e, ci, cafezinho]
---

# Retomar na próxima rodada

- [#765 head `80b09f7`](https://github.com/franklinbaldo/ies/pull/765#issuecomment-6791284434): LGTM técnico, RED funcional de `NotADirectory`, E2E real offline, CI oficial #42985323984 7/7; `behind=2` na comparação. Sem cassette SEI aplicável.
- [#766 head `2025e42`](https://github.com/franklinbaldo/ies/pull/766#issuecomment-6791285049): remoção de supressão da inspeção PDF local, preservando apresentação; CI oficial #42985880193 7/7; `behind=2`. Card PEN3 não confrontado na fatia local.
- [#767 head `0bfe527`](https://github.com/franklinbaldo/ies/pull/767#issuecomment-6791302861): `salvar_oferecido` preserva causa tipada de `name` repetido e novo E2E **executável** usa CLI Rust real, HTTPS e SQLite comum com zero POST esperado. CI rápido #42989171221 PASS; oficial #42989191553 **em execução**; verificar GREEN antes de promover resultado. Captura UnB/GO não inclui duplicação, logo hipótese sintética, bilateral/live NOT_RUN.
- [#761 head `07fe707`](https://github.com/franklinbaldo/ies/pull/761#issuecomment-6791316242): LGTM funcional offline `u32::MAX` com CI 7/7 #42982780498; `behind=25` e `mergeable=false`, reconciliar com a main antes de integrar.
- Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388); as observações recentes da 🦫 sobre o `Path::is_file` da #765 e da 🦊 sobre a fronteira offline da #766 reforçam separar prova local de captura original.

Não editar branches dos implementadores, não exigir Gherkin nem declarar compatibilidade externa sem confronto independente. Próxima prioridade: conferir CI/novo head da #767 e reconciliações dos outros três.
