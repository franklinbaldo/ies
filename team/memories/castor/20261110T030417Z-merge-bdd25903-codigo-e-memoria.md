---
type: AgentMemory
title: "BDD-259-03: preservar três arquivos ao reconciliar a main"
description: "A PR #622 foi reconciliada com 39 commits novos da main por merge verdadeiro sem conflitos de caminho; CI oficial foi reacionado, verificar no head final."
agent: castor
timestamp: "2026-11-10T03:04:17.616Z"
tags: [ies, anti-ies, andamento, bdd, merge, ci, sqlite, cassettes]
---

# Retomar

**Fato:** a [PR #622](https://github.com/franklinbaldo/ies/pull/622) estava atrás da main por 39 commits, mas a comparação confirmou zero sobreposição com os três arquivos do BDD-259-03. Montei merge de dois pais `4f3f3face`, preservando a árvore da main e somente os três blobs modificados da branch, sem rebase, force-push ou alteração de outros fluxos. A comparação então indicou `behind=0` e os mesmos três arquivos. Acionei nova execução do CI oficial via Draft → Ready; **não confundir execução disparada com gate aprovado**.

**Provas:** o último head anterior `857e7fb8` teve CI oficial 7/7; o novo ainda exige CI próprio. O 503 depois do POST permanece **sintético**; a inspeção de integridade UnB/GO (50 interações) não prova essa transição, e confronto semântico/live seguem NOT_RUN nesta fatia. Sem produção, credenciais, retry ou escrita externa.

**Conversa:** no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a 🐢 distinguiu perda de acesso à sessão de cancelamento do documento: efeitos parecidos na interface não equivalem ao mesmo estado persistido. Analogia útil para manter a causa tipada e o SQLite coerentes, não prova adicional.

**CONTINUAR:** após esta memória avançar a main, incorporar o commit documental à branch, conferir `behind=0` e obter CI oficial 7/7 no SHA definitivo antes de solicitar integração 🦅/🐘.
