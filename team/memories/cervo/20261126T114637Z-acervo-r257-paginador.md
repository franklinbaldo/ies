---
type: AgentMemory
title: "Acervo #895: paginador candidato e rápida verde"
description: "R257: Rustfmt corrigido; contrato de paginação do Todos ainda a confrontar com HTML original."
agent: cervo
timestamp: "2026-11-26T11:46:37Z"
tags: [acervo, paginacao, ci]
---

# Retomar

[PR #895](https://github.com/franklinbaldo/ies/pull/895) DRAFT, HEAD cee53173556568e1c1ddaf81d5251494d65772f6: remoção de linha vazia final conforme Rustfmt; [rápida 45813772269](https://github.com/franklinbaldo/ies/actions/runs/45813772269) SUCCESS, Rust/replay/E2E novos NOT_RUN.

O [Todos](https://github.com/franklinbaldo/todos/blob/caabe699bcf13b61b12dc5863638c8d90c0155c1/src/todos/sei/listagem_paginada.py) usa POST do formulário atual, hidden hdn*PaginaAtual zero-based e restaura a página inicial observada; hipótese coerente com as voltas nas cassettes PEN1/PEN2. Não portar fallback de campo ausente; confirmar form id, hidden e ação no HTML original e reaproveitar formulario_web/SubmitPorScript. Restauração não integra a coleta. Próxima fatia: transporte, paginação e CLI real. [Handoff](https://github.com/franklinbaldo/ies/pull/895#issuecomment-7191187049). Sem merge/live.
