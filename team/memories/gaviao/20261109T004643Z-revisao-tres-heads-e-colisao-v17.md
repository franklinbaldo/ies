---
type: AgentMemory
title: "Três heads revisados e colisão concorrente SQLite v17"
description: "Pareceres #520, #518 e #521 publicados com CI oficial 7/7; duas PRs disputam v17; integrar #518 antes da #521 e renumerar atribuição para v18."
agent: gaviao
timestamp: "2026-11-09T00:46:43Z"
tags: [revisao, bdd, anti-ies, sqlite, cassettes, ci, cafezinho]
---

# Ponto de retomada

**Fatos verificados:** permaneciam abertas apenas #518, #520 e #521 nesta consulta. Publiquei pareceres novos, um por head: [#520](https://github.com/franklinbaldo/ies/pull/520#issuecomment-6729570694), head `304a55e` (regressão da identidade de vínculo e preservação de outras relações); [#518](https://github.com/franklinbaldo/ies/pull/518#issuecomment-6729571044), `c33828c` (BDD-245-01, prévia via CLI real, TLS local e SQLite, 3 GET, 0 POST); [#521](https://github.com/franklinbaldo/ies/pull/521#issuecomment-6729572158), `f746c20` (BDD-264-01, prévia pela CLI real, 5 requisições, 1 POST de pesquisa e nenhum de atribuição). Os três heads têm CI rápido PASS e **7/7 jobs oficiais SUCCESS**, verificados pela API do Actions. Não duplicar pareceres nesses SHAs.

**Integração nova:** #491/v16 foi integrada; #518 já reconciliou a main e usa `0017_tipo_processo_simulado.sql`, mas #521 criou simultaneamente `0017_processo_atribuicao.sql`. Ambas incrementam `user_version=17`, registram include em `banco_cassettes.rs` e tocam testes; não integrar as duas sem conciliação. Prioridades das issues: #245=95, #264=94. Recomendei **#518/v17 primeiro, depois #521/v18**, por merge verdadeiro da main, preservando módulos e rotas e rodando novo CI completo no SHA resultante. #520 não possui migração e sua correção é independente. Um aparente mergeable=true individual não garante compatibilidade entre branches concorrentes.

**Escopo da prova:** #520 é regressão de motor Rust, não um E2E novo do BDD-258-03. #518 e #521 cumprem a prévia sintética do BDD, mas não as confirmações/escritas futuras. Consultei os cards PEN1 de remoção (25 interações), UnB de atribuição (24) e evidências UnB de criação; autores documentaram inspeções de ZIP via Actions. **Integridade/estrutura PASS ≠ confronto semântico independente NOT_RUN ≠ treinamento live NOT_RUN.** Não bloquear a evolução por ausência de REA/live.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), li os comentários mais recentes, inclusive 🦫 sobre identidade do vínculo e 🦊 sobre projeções que ocultam a não comutatividade. A analogia cinematográfica sugere procurar observação que separe dois roteiros causais, não apenas ampliar o mesmo fotograma; é inspiração, não evidência sobre o SEI.

**Próximo:** conferir se #518 integrou, examinar novo head da #521 com v18 e CI 7/7, e não confundir os dois cenários BDD de prévia com autorização de POST. Sem produção, escrita em SEI, implementação funcional, merge ou encerramento pelo Gavião.
