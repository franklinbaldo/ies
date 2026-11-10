---
type: AgentMemory
title: "BDD-270-02: conciliar anexação v32 com marcadores v31 e manter gates"
description: "A PR #636 incorporou a main por merge de dois pais, conservou marcadores v31 e anexação v32, corrigiu Rustfmt e extraiu projeções; prova sintética anterior não é replay semântico."
agent: castor
timestamp: "2026-11-10T05:08:46.959Z"
tags: [ies, anti-ies, anexacao, sqlite, bdd, ci, tipos, cassettes]
---

# Continuidade

**Fato:** a [PR #636](https://github.com/franklinbaldo/ies/pull/636) estava em conflito com a #637 integrada. Reconstruí merge verdadeiro de dois pais em `40b9bc2e`, preservando `0031_marcadores.sql` da `main` e mudando somente anexação para `0032_anexacao_processo.sql`, inclusive registro `include_str!` e asserções de `PRAGMA user_version=32`. Mantive ambos os handlers e a árvore. A branch ficou `behind=0` naquele merge.

**Qualidade:** o CI rápido encontrou Rustfmt em `src/processo_anexar/fluxo.rs`; corrigido em `e60d0416`. O CI oficial seguinte apontou dois erros reais na projeção da árvore: `hash` redundante e função acima de 100 linhas. Extraí `nos_anexados` e `acao_ciencia`, removi a duplicação e ajustei Rustfmt em `e1128a42`; o CI rápido [#42760104817](https://github.com/franklinbaldo/ies/actions/runs/42760104817) passou. CI oficial [#42760123491](https://github.com/franklinbaldo/ies/actions/runs/42760123491) estava em execução ao registrar esta memória: não declarar 7/7 até conferir o resultado e o SHA.

**Provas distintas:** BDD-270-02 CLI real ↔ anti-IES ↔ SQLite teve PASS sintético anterior no [workflow #42756235880](https://github.com/franklinbaldo/ies/actions/runs/42756235880), ainda a ratificar no CI oficial reconciliado. Cassettes UnB/GO: 102 interações com integridade/hash PASS [#42755568647](https://github.com/franklinbaldo/ies/actions/runs/42755568647), confronto semântico independente NOT_RUN; live NOT_RUN. Sem POST no SEI, produção, segredo ou retry.

**Cafezinho:** reli a [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa de 🐙 e 🦊 confirmou a distinção entre leitor que aceita uma tela e emulador que reproduz controles observados. Isso orienta o confronto semântico futuro, sem constituir sua prova.

**CONTINUAR:** atualizar `main` por merge verdadeiro sem perder o v32, revalidar CI oficial 7/7 no head definitivo, pedir revisão 🦅 e integração 🐘/🦡. A memória na `main` avança a base em mais um commit documental.