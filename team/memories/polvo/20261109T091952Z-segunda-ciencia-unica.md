---
type: AgentMemory
title: "Ciência documental: segunda tentativa pela CLI e identidade da atividade"
description: "Regressão de segunda ciência no BDD-271-02; inspeção GO? Não: UnB/UFRJ soma 77 interações íntegros; o vínculo ao usuário e live ainda precisam de prova."
agent: polvo
timestamp: "2026-11-09T09:19:52Z"
tags: [documento, ciencia, anti-ies, sqlite, bdd271, cassettes]
---

# Continuidade

- **Fato:** acrescentei à [PR #546](https://github.com/franklinbaldo/ies/pull/546) regressão da segunda invocação `ies documento ciente DOC --confirmar` contra CLI real, TLS local e mesmo SQLite. A hipótese testada é recusar pelo desaparecimento do botão antes do segundo GET mutador e manter o `id_atividade` original. O [CI #42592049860](https://github.com/franklinbaldo/ies/actions/runs/42592049860) foi 7/7 PASS no commit funcional `ff335da`; o head posterior precisa de CI final após sincronização.
- **Observação independente:** [inspeção GitHub Actions #42592112909](https://github.com/franklinbaldo/ies/actions/runs/42592112909) conferiu hashes de cassettes públicas UnB (39) e UFRJ (38), total 77 interações, com dois GETs `documento_ciencia` em cada. O arquivo temporário foi removido. Inspeção de integridade **não comprova** confronto semântico do aceite nem usuário associado.
- **Limites:** a ciência no SQLite é por documento/atividade, não associa usuário da sessão. A prova sintética é **parcial** para BDD-271-02; confronto semântico das respostas originais e live seguem NOT_RUN. Não acessar produção nem repetir GET potencialmente mutador.
- **Próximo passo:** sincronizar `main` com merge verdadeiro na PR, conferir gates do SHA exato, pedir revisão 🦅 e integração 🐘/🦡; conciliar migração SQL `0019_ciencia_documental.sql` com #526. A conversa [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) lembra que atividade encontrada e usuário que praticou o ato são testemunhas distintas, uma metáfora, não evidência do SEI.
