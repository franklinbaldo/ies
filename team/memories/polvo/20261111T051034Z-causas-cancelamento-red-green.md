---
type: AgentMemory
title: "Cancelamento documental: oito fontes tipadas ligadas com RED/GREEN"
description: "PR #756 demonstrou perda de Error::source() em sete recusas e no leitor do link de cancelamento; CI GREEN 7/7, cassettes GO/UFRJ inspecionadas, sem bilateral/live."
agent: polvo
timestamp: "2026-11-11T05:10:34Z"
tags: [documentos, cancelamento, ablacao, erros-tipados, red-green, cassette]
---

# Retomar

- **Fato:** [PR #756](https://github.com/franklinbaldo/ies/pull/756), subfatia de [#719](https://github.com/franklinbaldo/ies/issues/719), acrescenta apenas oito `#[source]` a causas já presentes em `ErroCancelar` (sete) e `ErroLinkCancelar` (uma); sem novos erros, subcódigos, mudanças de CLI ou anti-IES.
- **RED comprovado:** [CI #42962567882](https://github.com/franklinbaldo/ies/actions/runs/42962567882), **2079 PASS / 1 FAIL** na nova contraprova `erros_de_cancelamento_retêm_as_causas_tipadas`. **GREEN:** [CI #42963326094](https://github.com/franklinbaldo/ies/actions/runs/42963326094) **7/7 PASS** no head `084efa65`, após merge verdadeiro da main; CI rápido PASS. PR pronta para revisão 🦅 e integração 🐘/🦡, não mesclar por conta própria.
- **Cassette-first:** cards GO e UFRJ de `documento-mover-cancelar` inspecionados no [Actions #42962350114](https://github.com/franklinbaldo/ies/actions/runs/42962350114): **150 interações, hashes ZIP/JSON válidos**; não prova `Error::source()` nem a semântica de cancelamento. Confronto bilateral `NOT_RUN`; novo E2E específico CLI↔anti-IES↔SQLite `NOT_RUN`, porque a mudança é diagnóstica local; live `NOT_RUN`. Arquivo temporário da inspeção removido.
- **Continuação:** revisar a PR, verificar sincronia/CI do SHA se main avançar e retomar outras causas perdidas em #719 após integração, sem disputar branches.
- **Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388). A conversa da 🐢 sobre `Utf8Error` sem o vetor de bytes e a da 🦫 sobre `ErrorKind` lembram que encadear diagnóstico não autoriza carregar payload privado. É inspiração de segurança, não prova adicional.
