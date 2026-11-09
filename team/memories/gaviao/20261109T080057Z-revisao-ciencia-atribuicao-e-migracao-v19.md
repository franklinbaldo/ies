---
type: AgentMemory
title: "CI de ciência e atribuição; migração v19 concorrente"
description: "Pareceres #546/#547 demonstram falhas em teste E2E e Rustfmt; #548 já integrada; #526 precisa reconciliação de versão."
agent: gaviao
timestamp: "2026-11-09T08:00:57Z"
tags: [revisao, bdd, ci, anti-ies, sqlite, cassettes, cafezinho]
---

# Retomada

**Fatos:** novos pareceres por SHA [#546](https://github.com/franklinbaldo/ies/pull/546#issuecomment-6735124762) `87973b9` e [#547](https://github.com/franklinbaldo/ies/pull/547#issuecomment-6735125414) `7d45a67`. Na #546, CI 6/7: o novo BDD-271-02 excede a espera do proxy (`tests/aies_documento_cli.rs:112,371`); identificar o resultado da CLI/quantidade real de requisições antes de chamar PASS. A tabela `ciencia_documental` registra documento e atividade, ainda não o usuário exigido pelo BDD. Na #547, CI rápido falhou Rustfmt em `tests/aies_atribuicao_cli.rs:235`; a escrita sintética por CLI foi codificada, mas ainda falta gate válido no head. Nenhuma execução remota de escrita foi feita.

**Integração:** #548 `8dde0a8` foi mesclada durante a revisão (merge `26043fd`), com 7/7 no head, teste de URL de histórico revogada após cancelamento, **regressão HTTP/SQLite e não novo E2E da CLI**. #526 segue Draft no `5188493`, com migração `0018_processo_criado.sql` concorrente com v18 já integrada. #546 acrescenta v19 para ciência: a segunda a integrar entre criação e ciência precisa usar a próxima versão livre após merge verdadeiro, sem perder as duas funcionalidades.

**Provas:** cards de atribuição UnB (24 interações) e ciência UnB/UFRJ (39/38) consultados; não executar ou presumir confronto semântico independente nem live. Preservar PASS sintético apenas nos SHAs com CI efetivamente verde.

**Cafezinho:** reli [#388](https://github.com/franklinbaldo/ies/issues/388), 53 comentários. A conversa do mensageiro/partitura sugere distinguir endereço da evidência, evidência e prova causal; metáfora literária, não observação física nem SEI. Próxima rodada: rever heads novos e CI, sem duplicar pareceres.
