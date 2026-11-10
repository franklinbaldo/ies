---
type: AgentMemory
title: "BDD-ANOT-01: duplicação de link detectada por regressão transversal"
description: "PR #646: teste legado da tramitação refutou dois links de anotação; renderizador reconciliado com main, decisão OKF completa e causas Rust tipadas, CI reexecutado."
agent: polvo
timestamp: "2026-11-10T07:33:28.015Z"
tags: [anotacao, bdd638, anti-ies, cli, sqlite, ci, cassettes, erro-tipado]
---

# Retomada e fatos

- **Fatia e branch:** [PR #646](https://github.com/franklinbaldo/ies/pull/646), BDD-ANOT-01 em [#638](https://github.com/franklinbaldo/ies/issues/638). CLI real `ies processo anotar NUP --texto T` lê formulário da unidade e emite prévia, sem POST. O anti-IES deriva o link e o formulário do estado SQLite: `Some("")` é anotação observada vazia, `None` não é observação. A PR segue aberta/Ready.
- **Contraprova:** no [CI #42772884266](https://github.com/franklinbaldo/ies/actions/runs/42772884266), 2165 testes passaram e a regressão antiga `processo_enviar::tests_anti_ies::formulario_real_do_ies_tramita_e_confere_estado_do_anti_ies` falhou por `links_de_anotacoes_repetidos`; o renderizador publicava duas vezes `anotacao_registrar`. O gate `check_design` apontou seções e vínculos OKF ausentes.
- **Correção:** merge **verdadeiro de dois pais** [`8cf0b52`](https://github.com/franklinbaldo/ies/commit/8cf0b5295b7c88ffa21b77ea6c0229babc421f78), preservando a ação de anexação já integrada na main e produzindo um único link de anotação quando o processo está aberto na origem. Decisão OKF com alternativas, comparação, contrato, matriz e teste. Erro de anotação agora usa causa Rust `Causa` sem nova tabela `subcodigo()`; o motivo de situação não lida continua tipado.
- **Diagnóstico subsequente:** [CI #42775378323](https://github.com/franklinbaldo/ies/actions/runs/42775378323) passou nos seis jobs de replay/binário, mas falhou no `prek` só por Clippy: teste E2E de 103/100 linhas e campo `MotivoAbertoEm` não consumido. Apliquei justificativa local de `too_many_lines` no teste vertical e passei a apresentar o motivo tipado de situação, em vez de removê-lo. O [CI rápido #42775834070](https://github.com/franklinbaldo/ies/actions/runs/42775834070) passou no head [`3003223d`](https://github.com/franklinbaldo/ies/commit/3003223d8e30801bc699c143a28a6353213624ca). O [CI oficial #42775845614](https://github.com/franklinbaldo/ies/actions/runs/42775845614) estava **IN_PROGRESS** nesta leitura: não registrar PASS antes da conclusão.
- **Três provas:** E2E CLI real ↔ anti-IES ↔ SQLite = evidência sintética de testes, status do gate do SHA final em execução. Integridade das cassettes UnB (37) e Goiás (40) confirmada no [Actions #42772050666](https://github.com/franklinbaldo/ies/actions/runs/42772050666); confronto semântico IES e anti-IES ↔ cassette **NOT_RUN**. Treinamento/live **NOT_RUN**, sem acesso/escrita externa; não inferir POST a partir do GET.
- **Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e [registrei o aprendizado](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761649928): o leitor correto recusa controles repetidos; a forma observada inclui multiplicidade, além de presença/ausência.
- **CONTINUAR:** conferir CI oficial #42775845614 e possível drift de memórias na main, depois revisão Gavião/integração Elefante ou Texugo. Avançar ao BDD-ANOT-02 depois do merge; preservar confirmação fechada e exigência de escrita única com releitura causal. Handoff na [#638](https://github.com/franklinbaldo/ies/issues/638#issuecomment-6761669642).
