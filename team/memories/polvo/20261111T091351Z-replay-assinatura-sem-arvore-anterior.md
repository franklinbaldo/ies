---
type: AgentMemory
title: "Replay documental: ausência de árvore anterior não é caneta vazia"
description: "PR #768 protege a classificação de assinatura no replay; RED/GREEN e inspeção de duas cassettes no CI."
agent: polvo
timestamp: "2026-11-11T09:13:51Z"
tags: [documentos, replay, evidencias, erro-silencioso]
---

# Para retomar

A [PR #761](https://github.com/franklinbaldo/ies/pull/761) foi integrada por colega durante a rodada: conferir estado antes de reconciliar uma branch evita trabalho concorrente.

No [replay de assinatura da PR #768](https://github.com/franklinbaldo/ies/pull/768), `estado.antes.get(&d).cloned().unwrap_or_default()` equiparava **árvore anterior não observada** a **árvore observada sem caneta**. O novo teste Rust sintético falhou no [RED #42991524562](https://github.com/franklinbaldo/ies/actions/runs/42991524562) (2.085 aprovados, 1 contraprova falha); a correção preserva `None` separadamente, e o [GREEN #42992108059](https://github.com/franklinbaldo/ies/actions/runs/42992108059) concluiu 7/7. É correção do **relatório de replay**, não da CLI Web.

O [workflow de inspeção #42992950822](https://github.com/franklinbaldo/ies/actions/runs/42992950822) verificou ZIP e membro SHA para cards de escritas documentais UnB e PEN1, total 130 interações. Isto **não** comprova que falta uma árvore anterior nas capturas, nem fornece confronto semântico bilateral ou live. Teste usa a tela sintética pré-existente e nunca atribui corpo histórico inventado a cassette.

Depois de remover o arquivo temporário de inspeção, reconciliei `main` por merge verdadeiro [`53e773a`](https://github.com/franklinbaldo/ies/commit/53e773ae9bba93c847b7d9693791571028a67a21). Somente `src/documento_assinar/acervo.rs` está no diff. CI rápido desse SHA passou; conferir os **7 gates** do [CI oficial #42993134032](https://github.com/franklinbaldo/ies/actions/runs/42993134032) antes de pedir revisão e integração. A [issue #722](https://github.com/franklinbaldo/ies/issues/722) continua para outras ablações.
