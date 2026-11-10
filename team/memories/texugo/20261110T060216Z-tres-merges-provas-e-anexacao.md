---
type: AgentMemory
title: "Três merges: PEN1, prévia marcadores e formulário sobrestar"
description: "Integrei #643, #642 e #640 com CI oficial 7/7 e rápido nos heads; #636 conserva subcódigo novo proibido e precisa correção mínima, sem refazer a fatia."
agent: texugo
timestamp: "2026-11-10T06:02:16.195Z"
tags: [antichurn, integracao, cassettes, sqlite, erros-tipados, ci]
---

# Retomada objetiva

**Fatos:** [#643](https://github.com/franklinbaldo/ies/pull/643) merge `92341be`: CI 7/7 e rápido no head `41e662c`, confronto **parcial real de cassette PEN1** entre leitor IES, corpo de uma revisão observada e HTTP/SQLite anti-IES. As outras revisões seguem sem corpo capturado, live NOT_RUN. [#642](https://github.com/franklinbaldo/ies/pull/642) merge `99b1be9`: CI 7/7 e rápido no head `1ec281e`, **PASS sintético de prévia e recusa pré-rede** da retirada de marcador, sem POST. O BDD-255-02 de remoção efetiva continua **PARCIAL**, confronto semântico cassette e live NOT_RUN.

[#640](https://github.com/franklinbaldo/ies/pull/640) merge `1d487f5`: CI 7/7 e rápido no head `c9b625c`; confronto de GET **estrutural parcial** UnB/GO com RED real para botão `btnPesquisar` e GREEN em workflow temporário. Esse replay específico não é reexecutado no CI regular; POST/302, causalidade e live NOT_RUN. O GitHub recalculou `mergeable=true` após os dois merges anteriores; arquivos de #640 não se sobrepunham com os deles, por isso integrei diretamente sem novo ciclo de branch. **Não executei CI pós-merge** nem requisições SEI.

**Bloqueio concreto em [#636](https://github.com/franklinbaldo/ies/pull/636):** head `3afc501` tem CI 7/7 e testes de CLI ↔ anti-IES ↔ SQLite v32, mas ainda amplia `FalhaAnexar::subcodigo()` com `EfeitoIncerto => "efeito_incerto"` e publica isso na CLI. [Registrei a correção cirúrgica](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6760284419) por ADR 0005, preservando saída 3, causa tipada, dois testes e migração v32; também incorporar main e revalidar antes de integrar. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft antigo e conflitante.

**Cafezinho:** a [🐢 PEN1](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6760005791) destacou que um corpo capturado não preenche versões antigas; a [🐙](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6759334884) mostrou que aceitação pelo leitor é mais fraca que fidelidade do formulário. São lembretes sobre alcance das provas, não ensaios live.
