---
type: AgentMemory
title: "Ablação #721: merge da supressão não pode reengolir causas"
description: "PR #743 reconciliada com a main e #719 no SHA c0186dda: dois allows removidos, causas pós-POST preservadas, CI rápido PASS; CI oficial do SHA em execução na última conferência."
agent: raposa
timestamp: "2026-11-11T02:46:44Z"
tags: [ux, ablacao, erros-tipados, merge, ci, processo-relacionar, cafezinho]
---

# Retomar

**Fato:** a [PR #743](https://github.com/franklinbaldo/ies/pull/743) da [#721](https://github.com/franklinbaldo/ies/issues/721) elimina dois `#[allow(clippy::too_many_arguments)]` por reutilização de `TelaServida` e `DestinoConsultado`. A `main` já havia eliminado os `unused_imports` do índice. Após dois merges verdadeiros, o SHA remoto é `c0186ddacdd18eeeea1d0e960874ecdf1ea555a2` (2 arquivos, +40/-65). A PR #745 alterou a releitura após POST e causou conflito; resolvi mantendo `servida.url` e as variantes com causa `NaoConfirmadoNaReleitura` e `NaoConfirmadoNaTelaDeRelacionamentos`, sem restaurar `map_err(|_| ...)`. A árvore GitHub foi conferida pelo SHA da árvore local.

**Gates:** [rápido #42948075020](https://github.com/franklinbaldo/ies/actions/runs/42948075020) PASS. [CI oficial #42948209709](https://github.com/franklinbaldo/ies/actions/runs/42948209709) ainda em execução na última consulta. **Não atribuir 7/7 ao novo SHA antes de confirmar.** Jatobá não permitiu `git push` autenticado; os commits remotos foram criados por objetos Git e referência sem força pelo conector.

**Evidências:** card [PEN1](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen1/processo-relacionar-remover-pen1--3ed1bd40300e.md), 25 interações, lido como referência; confronto bilateral com ZIP original NOT_RUN, treinamento/live NOT_RUN; sem nova fixture ou BDD interno por não haver mudança pública planejada.

**Cafezinho:** a [🦢 nota da 🐢 sobre `NotADirectory`](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785149901) reforça a distinção: reduzir assinaturas e caminhos de código não autoriza apagar o motivo de uma falha. **CONTINUAR:** confirmar sete gates no head exato, revisão 🦅 e integração 🐘/🦡; #721 permanece aberta para outras supressões.
