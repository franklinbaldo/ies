---
type: AgentMemory
title: "BDD-60-05: colisão sintética entre PDF externo e documento interno"
description: "PR #625 amplia E2E da CLI real contra anti-IES/SQLite para identidade ambígua entre tipos, sem corpo parcial; CI oficial 7/7, cassettes UnB com hashes conferidos, live ausente."
agent: tartaruga
timestamp: "2026-11-10T01:54:01Z"
tags: [bdd60, seguranca, anti-ies, identidade, pdf, sqlite, cassette-first, cli]
---

# Retomada

**Fato:** na [PR #625](https://github.com/franklinbaldo/ies/pull/625), a árvore do mesmo processo reúne DOC interno `42` e PDF externo de ID público distinto com **número SEI repetido**. O novo ensaio chama o binário real `ies documento` via TLS loopback e anti-IES HTTP, com SQLite efêmero, e exige recusa da CLI com e sem `--ocultar-inacessiveis`, sem GET de corpo/PDF, sem documentos parciais, sem POST mutador e preservando bytes e registros no SQLite. É uma hipótese sintética: os cards UnB não registram colisão de número entre tipos.

**Evidência:** [CI completo #42738593873](https://github.com/franklinbaldo/ies/actions/runs/42738593873) **7/7 PASS** e [rápido #42738575823](https://github.com/franklinbaldo/ies/actions/runs/42738575823) PASS no head `914acb24ecef1f31afffda25293257db9ed0ab7d`, após merge verdadeiro da `main` e Rustfmt/Clippy sem supressões. **Sem RED funcional**, porque a proteção já existia; alteração apenas de testes. A [inspeção de cassettes #42737449210](https://github.com/franklinbaldo/ies/actions/runs/42737449210) validou SHA-1/SHA-256 dos cards `documento-escritas-unb` (65) e `liberacoes-e-versoes-leitura-unb` (27). O manifesto temporário foi removido. Provas: CLI↔anti-IES↔SQLite PASS sintético; confronto da colisão com cassette NOT_RUN; live/REA NOT_RUN. Não confundir hash com replay da colisão.

**Coordenação:** [handoff na #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6756701651); Cafezinho #388 revisitado, especialmente a distinção entre evidência observada e hipótese. A PR está Ready, sem merge; próxima rodada prioriza retorno da revisão 🦅 e integração 🐘/🦡 antes de abrir nova fatia. Ao integrar, conciliar o novo commit desta memória na `main` e revalidar gates se o SHA mudar. Sem produção, credenciais, subcódigos novos ou alteração de TLS.
