---
type: AgentMemory
title: "R299: #881 intacta; formulário processual e action da mesma tela"
description: "Comparação de blobs e oráculo Todos para a próxima fatia de inclusão processual."
agent: polvo
timestamp: 2026-11-28T07:44:11Z
tags: [ies, bloco, cassette, formulario]
---

# Retomar

- #881 DRAFT, HEAD d6471ad4; 7/7 blobs do candidato R298 iguais entre 46a78c23 e main 652a280a; zero paths comuns com #922 (14) e #908 (17). #901 encerrada sem merge; seguir R307, #922→#908, sem novo WIP ou dispatch.
- **Achado:** `tests/test_bloco_de_processos_010.py` do Todos deriva a action e `frmVisualizar` de `arvore_visualizar` ordem=1 no cenário PEN1; o código faz isso antes de abrir a lupa. Confirmar no ZIP original. Se confirmada, reutilizar `derivar_submissao_com_action` da mesma tela, sem abstração multi-tela; faltam ações tipadas e efeito processual. Oferta geral não comprova elegibilidade do bloco na lupa.
- [Recibo #881](https://github.com/franklinbaldo/ies/pull/881#issuecomment-7236151082). Rust/E2E/replay/prek/live NOT_RUN, sem merge ou alteração de HEAD. Não afirmar paridade. Revalidar base e revisão excepcional antes de integrar.
