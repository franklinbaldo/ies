---
type: AgentMemory
title: "Inclusão documental: vínculo checkbox–documento não demonstrado"
description: "Na PR #701, o formulário aceita vários chkDocumentosItemN mas o POST escolhe sempre Item0; contraprova de dois itens registrada para a Raposa sem alegar falha live."
agent: polvo
timestamp: "2026-11-10T20:47:58Z"
tags: [blocos, documentos, checkbox, anti-ies, bdd, prova-causal]
---

# Para retomar

- **Fato no código:** `ler_tela_de_escolha` em `src/bloco_incluir.rs` coleta títulos de todas as caixas `chkDocumentosItemN`, mas `confirmar_documento` usa sempre `chkDocumentosItem0`. `derivar_submissao` valida nome publicado, não o vínculo daquele campo com o `title` e `value` pretendidos. O anti-IES da [PR #701](https://github.com/franklinbaldo/ies/pull/701) emite só uma caixa, de índice zero.
- **Hipótese falsificadora, não bug live comprovado:** se Item0 aponta para DOC-A e Item1 para DOC-B pedido, a prévia permite DOC-B enquanto o POST seleciona Item0; `hdnDocumentosItensSelecionados` ainda é reenviado literalmente. Registrei [contraprova proposta na PR](https://github.com/franklinbaldo/ies/pull/701#issuecomment-6778861223), pedindo vínculo explícito ou recusa antes do POST e teste real CLI↔anti-IES↔SQLite de dois itens. Não alterei a branch da 🦊.
- **Provas:** [CI oficial #42897607950](https://github.com/franklinbaldo/ies/actions/runs/42897607950) terminou 7/7 success no head `a955483` da #701, mas não cobre o caso proposto. Integridade/inventário UnB anteriormente PASS; confronto bilateral da seleção e transição NOT_RUN, live NOT_RUN. A #701 seguia aberta na última consulta. Não houve SEI externo nem gravação em produção.
- **Ponto de retomada:** acompanhar a resposta da 🦊/🦅, não disputar BDD-381-02; após a integração, verificar se BDD-381-03 ficou livre e atacar idempotência e recusas canônicas. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6778796535): a distinção entre 302 e associação comprovada reforça exigir identidade da caixa antes do envio.
