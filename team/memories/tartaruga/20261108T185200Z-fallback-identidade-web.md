---
type: AgentMemory
title: "Identidade Web: duplicação não autoriza fallback"
description: "PR #493 corrige fallback de id_documento duplicado para id_protocolo; RED HTTP 200 indevido em CI, inspeção estrutural de cassettes UnB aprovada e BDD-60-04 acrescentado."
agent: tartaruga
timestamp: "2026-11-08T18:52:00Z"
tags: [navegacao, anti-ies, bdd, documento, cassette, identidade]
---

# Retomada

**Fato:** em [RED #42511730551](https://github.com/franklinbaldo/ies/actions/runs/42511730551), o GET com `id_documento` duplicado e `id_protocolo` alternativo entregou HTTP 200 da árvore de outro documento. A [PR #493](https://github.com/franklinbaldo/ies/pull/493) corrige a precedência; acrescentei também teste de CLI real para **BDD-60-04**, propriedades não oferecidas na barra devem permanecer nulas e sem GET de cadastro. CI do head corrigido precisa ser conferido antes da integração.

**Observação:** [inspeção Actions #42511441855](https://github.com/franklinbaldo/ies/actions/runs/42511441855) confirmou hashes de duas cassettes UnB, 22 interações; não houve confronto semântico independente nem live. Sem novo subcódigo, schema ou alteração de TLS. A conversa no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) reforça que ausência não equivale a observação negativa.
