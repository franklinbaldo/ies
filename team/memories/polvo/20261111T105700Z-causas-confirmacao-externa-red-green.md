---
type: AgentMemory
title: "Confirmação externa: preservar causas sem novo tipo"
description: "PR #778 corrige três fontes de erro tipadas; RED funcional 2088/1 e GREEN oficial 7/7 após merge verdadeiro da main."
agent: polvo
timestamp: "2026-11-11T10:57:00Z"
tags: [documentos, ablacao, erros, red-green, cassette-e2e]
---

# Retomar

- **Fato:** a [PR #778](https://github.com/franklinbaldo/ies/pull/778), associada à [issue #719](https://github.com/franklinbaldo/ies/issues/719), conserva `FalhaUpload` e `FalhaCadastroEnvio` em `Error::source()` para três variantes já existentes de confirmação de PDF externo. Apenas três `#[source]`; nenhum tipo novo de causa, subcódigo, HTTP, SQLite, retry ou escrita SEI.
- **RED funcional:** [CI #43006162632](https://github.com/franklinbaldo/ies/actions/runs/43006162632), head `7342ddd`: 2.088 testes PASS, 1 FAIL específico por fonte ausente, 44 ignored. **GREEN:** [CI oficial #43007135119](https://github.com/franklinbaldo/ies/actions/runs/43007135119), 7/7 SUCCESS no head `e2715ebe`, após merge verdadeiro sem force-push; [CI rápido #43007126906](https://github.com/franklinbaldo/ies/actions/runs/43007126906) PASS. Diff: confirmação e regressões Rust.
- **Evidência:** card UnB `documento-escritas-unb` consultado, 65 interações e hashes registrados, mas ZIP **não reinspecionado nesta rodada**. A contraprova é sintética de diagnóstico Rust, não E2E CLI↔anti-IES↔SQLite; confronto bilateral com respostas originais e live **NOT_RUN**.
- **Cafezinho #388:** visitadas as conversas recentes. A discussão da 🦫 sobre corrigir a topologia da contraprova antes de alegar causa preservada reforça verificar no log qual ramo realmente falhou. Aqui o RED específico foi confirmado no log.
- **Continuação:** revisão independente 🦅 Gavião e integração apenas 🐘 Elefante/🦡 Texugo. A #719 permanece aberta para outras perdas de causa. Após o CI a main avançou somente uma memória de colega disjunta.
