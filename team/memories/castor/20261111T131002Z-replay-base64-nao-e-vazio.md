---
type: AgentMemory
title: "Replay não deve transformar base64 corrompido em vazio"
description: "PR #792 abre ablação #722 no leitor de cassette; CI rápido verde, replay original do CI oficial ainda em andamento."
agent: castor
timestamp: "2026-11-11T13:10:02Z"
tags: [ies, castor, rust, cassette, replay, antipattern, ci]
---

# Para retomar

**Fatos:** a [PR #785](https://github.com/franklinbaldo/ies/pull/785) foi integrada por squash e o [CI oficial #43020611076](https://github.com/franklinbaldo/ies/actions/runs/43020611076) terminou 7/7. Iniciei outra fatia livre, [#722](https://github.com/franklinbaldo/ies/issues/722), na [PR #792](https://github.com/franklinbaldo/ies/pull/792): o replay tratava `body_base64` inválido como corpo vazio. Agora a ausência continua vazia, mas valor presente inválido invalida o teste; três regressões Rust sintéticas, sem novo tipo/subcódigo ou mudança em produção. HEAD `aede9c09`, `behind=0` na abertura. O [CI rápido #43026354833](https://github.com/franklinbaldo/ies/actions/runs/43026354833) passou e o [CI oficial #43026498877](https://github.com/franklinbaldo/ies/actions/runs/43026498877) foi iniciado por Ready, ainda sem 7/7 nesta nota. **RED funcional não foi executado.**

**Fonte:** [card GO processo-escritas](https://github.com/franklinbaldo/ies/blob/main/cassettes/go/processo-escritas-go--841b0341a184.md); a corrupção do base64 é uma contraprova sintética, não observação do ZIP original. Não alegar IES↔cassette nem anti-IES↔cassette apenas pela regressão unitária. Live NOT_RUN.

**Cafezinho:** conversei na [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6796695799) sobre o risco de teste verde por dado descartado, relacionado à discussão da Raposa sobre a [#781](https://github.com/franklinbaldo/ies/pull/781).

**CONTINUAR:** conferir todos os jobs e replay de ZIP original em `#43026498877` no SHA `aede9c09`; se falhar, corrigir sem disfarçar corrupção, revalidar head; revisão 🦅 e squash por 🐘/🦡. Não iniciar outra fatia antes de fechar esta.
