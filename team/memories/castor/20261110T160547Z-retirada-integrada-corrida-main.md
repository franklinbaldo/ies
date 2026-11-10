---
type: AgentMemory
title: "Retirada BDD-348-02 integrada; reconciliação concorrente não altera prova"
description: "PR #678 integrada na main pelo Texugo, CI oficial 7/7 no head bb50ac9; 1 POST e releitura sintéticos, política #681 saída 1; replay bilateral e live pendentes."
agent: castor
timestamp: "2026-11-10T16:05:47.204Z"
tags: [ies, anti-ies, acompanhamento, bdd, rust, ci, cassettes, handoff]
---

# Retomar

**Fato:** a [PR #678](https://github.com/franklinbaldo/ies/pull/678) integrou na main pelo 🦡 no [merge 6d5a3fa](https://github.com/franklinbaldo/ies/commit/6d5a3fa76c1b161c9bd9609a7ee79084bdee38eb), durante a reconciliação final da branch pelo 🦫. O [CI oficial #42848998633](https://github.com/franklinbaldo/ies/actions/runs/42848998633) passou 7/7 e o [rápido #42848970403](https://github.com/franklinbaldo/ies/actions/runs/42848970403) passou no head integrado `bb50ac9`. Minha reconciliação adicional posterior à integração foi um merge verdadeiro `431c39e` apenas na branch, **não** o commit validado nem o merge da main. Conferi o estado de PR após o movimento concorrente. Nenhuma outra alteração produtiva foi feita nesta rodada.

**BDD-348-02:** CLI Rust real → HTTPS local → anti-IES/SQLite confirma a retirada de G com **um POST** e segunda leitura preservando H. Saída humana corrigida e 503 sintético sem retry. A [decisão do proprietário #681](https://github.com/franklinbaldo/ies/issues/681) define **saída 1 / `nao_confirmado`**, sobrepondo o texto anterior de saída 3; o 🐘 deve reconciliar o BDD canônico. Deixei o [handoff na #348](https://github.com/franklinbaldo/ies/issues/348#issuecomment-6772358198) e liberei o emoji de trabalho no título.

**Limites:** nível 1 **PASS sintético** no SHA `bb50ac9`; 97 interações UnB tiveram apenas hashes/estrutura verificados em inspeção anterior, **ambos↔cassette semântico NOT_RUN**; **treinamento/live NOT_RUN**. Não inferir operação real da transição 302/503.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388) e li as discussões sobre estados observados versus ausência de observação. A releitura causal no SQLite é prova de consistência interna, não evidência de fidelidade externa.

**CONTINUAR:** inspecionar a prioridade livre #60 (BDD-60-05 parcialmente coberto, muitos E2E já integrados) e evitar adicionar testes redundantes; escolher subcaso mensurável antes de abrir nova PR. Não refazer #348 nem reabrir branch integrada.
