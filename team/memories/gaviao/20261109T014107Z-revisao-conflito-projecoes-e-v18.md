---
type: AgentMemory
title: "Revisão de quatro heads: projeções concorrentes e SQLite v18"
description: "Pareceres #524/#523/#521/#526; #525 já integrada; #521 e #526 precisam conciliar a navegação e a migração v18, com CI e cobertura BDD específicos."
agent: gaviao
timestamp: "2026-11-09T01:41:07Z"
tags: [revisao, bdd, anti-ies, sqlite, ci, conflitos, cafezinho]
---

# Retomar na próxima rodada

**Fatos:** publiquei um parecer por head aberto: [#524](https://github.com/franklinbaldo/ies/pull/524#issuecomment-6730221224) `8035f70`, regressão técnica para preservar vínculos ao adicionar, CI oficial 7/7; [#523](https://github.com/franklinbaldo/ies/pull/523#issuecomment-6730221568) `4eb6ebc`, motivo opaco de cancelamento, CLI real↔anti-IES↔SQLite PASS sintético e CI 7/7; [#521](https://github.com/franklinbaldo/ies/pull/521#issuecomment-6730221905) `ea0c268`, BDD-264-01 prévia da atribuição PASS sintético e CI 7/7 mas `mergeable_state=dirty` após [#525](https://github.com/franklinbaldo/ies/pull/525) integrar; [#526](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6730222287) `6fbd728`, implementação de criação com teste da CLI ainda sem CI nesse SHA, `mergeable_state=dirty`, BDD-245-02 sem segunda invocação `ies processo NUP`.

**Integração:** #525 extraiu navegação/árvore para `processo_envio/navegacao.rs`; #521 extraiu outra árvore e altera assinatura da barra; #526 edita a árvore ainda no arquivo antigo. Necessário merge verdadeiro e preservar as três capacidades. **#521 e #526 propõem duas migrações `0018` distintas**; caminho preferido: concluir #521/v18, deslocar #526 para v19 e revalidar CI por SHA. Não confundir a correção de Rustfmt do head anterior da #526 com CI PASS do head novo.

**Evidências:** inspecionei cards PEN1 de criação e UnB/GO de atribuição, runs de integridade dos autores e CI. Sempre distinguir PASS sintético, hash/estrutura de cassette, confronto semântico independente NOT_RUN e treinamento NOT_RUN. Não houve REA, acesso SEI, merge, fechamento ou implementação pelo Gavião.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), com 46 comentários. A ideia do contrafólio/contracampo continua útil como analogia da verificação de efeitos por observação nova, não como evidência sobre SEI ou física da prensa.

**Próximo:** verificar heads de #521/#526 após conciliação, CI 7/7 atualizado e manutenção de `pode_incluir_documento`, atribuição e criação sem duplicar a árvore; acompanhar integração de #524/#523 sem pedir retrabalho.
