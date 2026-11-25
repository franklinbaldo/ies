---
type: AgentMemory
title: "Árvore mista: próximo índice vem do SQLite, não do JavaScript renderizado"
description: "PR #710 troca contagem textual de nós por índice da projeção de documentos, pasta e anexações; HTTP sintético de dois PDFs em avaliação, CI completo ainda não executado."
agent: tartaruga
timestamp: "2026-11-10T22:42:47Z"
tags: [anti-ies, bdd60, sqlite, navegacao, projecao, cassette-first, ci]
---

# Para retomar

**Fato:** a [PR #706](https://github.com/franklinbaldo/ies/pull/706) foi integrada. No BDD-60-05 / [issue #496](https://github.com/franklinbaldo/ies/issues/496), a árvore mista ainda contava nós procurando `Nos[` no HTML recém-gerado, embora documentos, PASTA/AGUARDE e anexações já tenham origem tipada no SQLite.

**Fatia:** [PR #710](https://github.com/franklinbaldo/ies/pull/710), draft. Extraí `nos_e_proximo_indice` da navegação e usei a posição derivada do estado para inserir PDFs. Teste novo HTTP de pasta e PDF inicial → cadastro de segundo PDF no mesmo SQLite → nova árvore → POST 405 → releitura imutável. Não inventei cenário BDD novo.

**Evidência:** CI rápido [#42916714198](https://github.com/franklinbaldo/ies/actions/runs/42916714198) PASS no SHA `1fbf7a0c` (Rustfmt corrigido após falha anterior). CI completo, RED funcional, confronto da variante com cassette e live **NOT_RUN** nesta hora; não converter inspeção anterior dos cards UnB (75 HTTP, [#42906730834](https://github.com/franklinbaldo/ies/actions/runs/42906730834)) em prova da transição sintética. A CLI real ainda depende dos E2Es preexistentes para regressão.

**Próximo passo:** incorporar `main` por merge verdadeiro de dois pais, executar CI completo via draft→ready e corrigir qualquer falha; só pedir revisão 🦅 após gates verdes, sem merge por esta rotina. Sem credenciais, produção, subcódigos ou TLS relaxado.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a reflexão da 🦊 sobre pertencimento positivo que não depende de um botão ainda ofertado. Na mesma linha, uma posição de nó deriva do estado, não de contar linhas de uma representação.
